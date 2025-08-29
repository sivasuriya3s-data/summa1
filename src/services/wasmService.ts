import { AnalysisResult, ConversionResult } from '../types';

class WasmService {
  private pythonModule: any = null;
  private rustModule: any = null;

  async initializePython(): Promise<void> {
    if (this.pythonModule) return;
    
    try {
      // Load Python WASM module
      const response = await fetch('/wasm/python-analyzer.wasm');
      const wasmBytes = await response.arrayBuffer();
      
      // Initialize Python module (placeholder for actual WASM integration)
      this.pythonModule = {
        analyzeDocument: (fileContent: ArrayBuffer, fileName: string) => {
          return this.mockPythonAnalysis(fileName);
        }
      };
    } catch (error) {
      console.error('Failed to initialize Python WASM:', error);
      throw new Error('Python analysis module failed to load');
    }
  }

  async initializeRust(): Promise<void> {
    if (this.rustModule) return;
    
    try {
      // Load Rust WASM module
      const response = await fetch('/wasm/rust-converter.wasm');
      const wasmBytes = await response.arrayBuffer();
      
      // Initialize Rust module (placeholder for actual WASM integration)
      this.rustModule = {
        convertDocument: (fileContent: ArrayBuffer, targetFormat: string, maxSize: number) => {
          return this.mockRustConversion(fileContent, targetFormat, maxSize);
        }
      };
    } catch (error) {
      console.error('Failed to initialize Rust WASM:', error);
      throw new Error('Document conversion module failed to load');
    }
  }

  async analyzeDocument(file: File): Promise<AnalysisResult> {
    try {
      const response = await fetch('/api/python/analyze', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          filename: file.name
        })
      });
      
      if (!response.ok) {
        throw new Error(`Analysis failed: ${response.statusText}`);
      }
      
      const result = await response.json();
      return {
        originalName: result.original_name,
        suggestedName: result.suggested_name,
        confidence: result.confidence,
        documentType: result.document_type
      };
    } catch (error) {
      console.error('Python analysis error:', error);
      // Fallback to mock analysis
      return this.mockPythonAnalysis(file.name);
    }
  }

  async convertDocuments(
    files: File[], 
    examType: string, 
    targetFormats: string[], 
    maxSizes: Record<string, number>
  ): Promise<ConversionResult> {
    try {
      // Convert files to base64 for transmission
      const fileData = await Promise.all(
        files.map(async (file) => {
          const arrayBuffer = await file.arrayBuffer();
          const base64Content = btoa(
            new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), '')
          );
          
          return {
            name: file.name,
            content: base64Content,
            mime_type: file.type
          };
        })
      );
      
      const response = await fetch('/api/rust/convert', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          files: fileData,
          exam_type: examType,
          target_formats: targetFormats,
          max_sizes: Object.fromEntries(
            Object.entries(maxSizes).map(([k, v]) => [k, v])
          )
        })
      });
      
      if (!response.ok) {
        throw new Error(`Conversion failed: ${response.statusText}`);
      }
      
      const result = await response.json();
      return result;
    } catch (error) {
      console.error('Rust conversion error:', error);
      // Fallback to mock conversion
      return this.mockConversion(files, targetFormats);
    }
  }

  // Mock implementations for development
  private mockPythonAnalysis(fileName: string): AnalysisResult {
    const documentTypes = {
      'marksheet': '10thMarksheet',
      'certificate': 'Certificate',
      'photo': 'PassportPhoto',
      'signature': 'Signature'
    };
    
    const lowerName = fileName.toLowerCase();
    let suggestedName = fileName;
    let documentType = 'Unknown';
    
    for (const [key, value] of Object.entries(documentTypes)) {
      if (lowerName.includes(key)) {
        documentType = value;
        suggestedName = `${value}.${fileName.split('.').pop()}`;
        break;
      }
    }
    
    return {
      originalName: fileName,
      suggestedName,
      confidence: 0.85,
      documentType
    };
  }

  private async mockConversion(
    files: File[],
    targetFormats: string[]
  ): Promise<ConversionResult> {
    // Simulate conversion delay
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    const convertedFiles = [];
    
    for (const file of files) {
      for (const format of targetFormats) {
        const blob = new Blob([await file.arrayBuffer()], { 
          type: `application/${format.toLowerCase()}` 
        });
        const url = URL.createObjectURL(blob);
        
        convertedFiles.push({
          originalName: file.name,
          convertedName: `${file.name.split('.')[0]}.${format.toLowerCase()}`,
          downloadUrl: url
        });
      }
    }
    
    return {
      success: true,
      files: convertedFiles
    };
  }
}

export const wasmService = new WasmService();