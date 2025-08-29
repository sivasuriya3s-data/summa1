import re
import json
from typing import Dict, List, Tuple

class DocumentAnalyzer:
    def __init__(self):
        self.document_patterns = {
            'marksheet': [
                r'marksheet|mark\s*sheet',
                r'grade\s*report',
                r'academic\s*record',
                r'transcript'
            ],
            'certificate': [
                r'certificate',
                r'diploma',
                r'degree',
                r'qualification'
            ],
            'photo': [
                r'photo',
                r'photograph',
                r'image',
                r'picture'
            ],
            'signature': [
                r'signature',
                r'sign',
                r'autograph'
            ],
            'identity': [
                r'aadhar|aadhaar',
                r'pan\s*card',
                r'voter\s*id',
                r'passport',
                r'driving\s*license'
            ],
            'category': [
                r'caste\s*certificate',
                r'category\s*certificate',
                r'reservation\s*certificate',
                r'obc|sc|st|ews'
            ]
        }
        
        self.class_patterns = {
            '10th': [r'10th|tenth|class\s*10|x\s*class'],
            '12th': [r'12th|twelfth|class\s*12|xii\s*class'],
            'graduation': [r'graduation|bachelor|b\.?tech|b\.?sc|b\.?com|b\.?a'],
            'post_graduation': [r'post\s*graduation|master|m\.?tech|m\.?sc|m\.?com|m\.?a']
        }

    def analyze_filename(self, filename: str) -> Dict:
        """Analyze filename and suggest a better name based on content patterns"""
        filename_lower = filename.lower()
        
        # Remove file extension for analysis
        name_without_ext = filename_lower.rsplit('.', 1)[0]
        file_ext = filename.rsplit('.', 1)[1] if '.' in filename else ''
        
        # Detect document type
        document_type = self._detect_document_type(name_without_ext)
        
        # Detect class/education level
        education_level = self._detect_education_level(name_without_ext)
        
        # Generate suggested name
        suggested_name = self._generate_suggested_name(
            document_type, education_level, file_ext
        )
        
        # Calculate confidence score
        confidence = self._calculate_confidence(name_without_ext, document_type, education_level)
        
        return {
            'original_name': filename,
            'suggested_name': suggested_name,
            'document_type': document_type,
            'education_level': education_level,
            'confidence': confidence
        }

    def _detect_document_type(self, text: str) -> str:
        """Detect the type of document based on filename patterns"""
        for doc_type, patterns in self.document_patterns.items():
            for pattern in patterns:
                if re.search(pattern, text, re.IGNORECASE):
                    return doc_type
        return 'unknown'

    def _detect_education_level(self, text: str) -> str:
        """Detect education level from filename"""
        for level, patterns in self.class_patterns.items():
            for pattern in patterns:
                if re.search(pattern, text, re.IGNORECASE):
                    return level
        return 'unknown'

    def _generate_suggested_name(self, doc_type: str, edu_level: str, file_ext: str) -> str:
        """Generate a standardized filename"""
        name_parts = []
        
        if edu_level != 'unknown':
            if edu_level == '10th':
                name_parts.append('10th')
            elif edu_level == '12th':
                name_parts.append('12th')
            elif edu_level == 'graduation':
                name_parts.append('Graduation')
            elif edu_level == 'post_graduation':
                name_parts.append('PostGraduation')
        
        if doc_type != 'unknown':
            if doc_type == 'marksheet':
                name_parts.append('Marksheet')
            elif doc_type == 'certificate':
                name_parts.append('Certificate')
            elif doc_type == 'photo':
                name_parts.append('Photo')
            elif doc_type == 'signature':
                name_parts.append('Signature')
            elif doc_type == 'identity':
                name_parts.append('IdentityProof')
            elif doc_type == 'category':
                name_parts.append('CategoryCertificate')
        
        if not name_parts:
            name_parts.append('Document')
        
        suggested_name = ''.join(name_parts)
        return f"{suggested_name}.{file_ext}" if file_ext else suggested_name

    def _calculate_confidence(self, text: str, doc_type: str, edu_level: str) -> float:
        """Calculate confidence score for the analysis"""
        confidence = 0.0
        
        # Base confidence for document type detection
        if doc_type != 'unknown':
            confidence += 0.5
        
        # Additional confidence for education level detection
        if edu_level != 'unknown':
            confidence += 0.3
        
        # Bonus for multiple pattern matches
        total_matches = 0
        for patterns in self.document_patterns.values():
            for pattern in patterns:
                if re.search(pattern, text, re.IGNORECASE):
                    total_matches += 1
        
        for patterns in self.class_patterns.values():
            for pattern in patterns:
                if re.search(pattern, text, re.IGNORECASE):
                    total_matches += 1
        
        if total_matches > 1:
            confidence += 0.2
        
        return min(confidence, 1.0)

# Main function for WASM integration
def analyze_document(filename: str) -> str:
    """Main entry point for document analysis"""
    analyzer = DocumentAnalyzer()
    result = analyzer.analyze_filename(filename)
    return json.dumps(result)

# Test function
if __name__ == "__main__":
    analyzer = DocumentAnalyzer()
    
    test_files = [
        "10marksheet.pdf",
        "12th_certificate.jpg",
        "graduation_degree.pdf",
        "photo.jpeg",
        "signature.png",
        "caste_certificate.pdf"
    ]
    
    for filename in test_files:
        result = analyzer.analyze_filename(filename)
        print(f"File: {filename}")
        print(f"Analysis: {json.dumps(result, indent=2)}")
        print("-" * 50)