export interface ExamConfig {
  name: string;
  formats: string[];
  maxSizes: {
    [format: string]: number; // in bytes
  };
  requirements: string[];
}

export interface FileItem {
  id: string;
  file: File;
  name: string;
  originalName: string;
  size: number;
  type: string;
  status: 'pending' | 'analyzing' | 'converting' | 'success' | 'error';
  progress: number;
  convertedUrl?: string;
  error?: string;
}

export interface ConversionResult {
  success: boolean;
  files: {
    originalName: string;
    convertedName: string;
    downloadUrl: string;
  }[];
  error?: string;
}

export interface AnalysisResult {
  originalName: string;
  suggestedName: string;
  confidence: number;
  documentType: string;
}