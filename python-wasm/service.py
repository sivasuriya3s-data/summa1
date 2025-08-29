from flask import Flask, request, jsonify
from flask_cors import CORS
import json
from analyzer import DocumentAnalyzer

app = Flask(__name__)
CORS(app)

analyzer = DocumentAnalyzer()

@app.route('/health', methods=['GET'])
def health_check():
    return jsonify({'status': 'healthy', 'service': 'python-analyzer'})

@app.route('/analyze', methods=['POST'])
def analyze_document():
    try:
        data = request.get_json()
        filename = data.get('filename')
        
        if not filename:
            return jsonify({'error': 'Filename is required'}), 400
        
        result = analyzer.analyze_filename(filename)
        return jsonify(result)
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

@app.route('/batch-analyze', methods=['POST'])
def batch_analyze():
    try:
        data = request.get_json()
        filenames = data.get('filenames', [])
        
        if not filenames:
            return jsonify({'error': 'Filenames array is required'}), 400
        
        results = []
        for filename in filenames:
            result = analyzer.analyze_filename(filename)
            results.append(result)
        
        return jsonify({'results': results})
        
    except Exception as e:
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8001, debug=False)