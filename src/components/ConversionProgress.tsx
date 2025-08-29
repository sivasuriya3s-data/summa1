import React from 'react';
import { FileText, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';
import { FileItem } from '../types';

interface ConversionProgressProps {
  files: FileItem[];
}

export default function ConversionProgress({ files }: ConversionProgressProps) {
  if (files.length === 0) return null;

  const getStatusIcon = (status: FileItem['status']) => {
    switch (status) {
      case 'analyzing':
        return <Loader2 className="w-4 h-4 animate-spin text-blue-500" />;
      case 'converting':
        return <Loader2 className="w-4 h-4 animate-spin text-purple-500" />;
      case 'success':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      default:
        return <FileText className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusText = (status: FileItem['status']) => {
    switch (status) {
      case 'analyzing':
        return 'Analyzing content...';
      case 'converting':
        return 'Converting format...';
      case 'success':
        return 'Conversion complete';
      case 'error':
        return 'Conversion failed';
      default:
        return 'Pending';
    }
  };

  return (
    <div className="mt-6">
      <h3 className="text-lg font-semibold text-gray-800 mb-4">
        Conversion Progress
      </h3>
      
      <div className="space-y-3">
        {files.map((fileItem) => (
          <div key={fileItem.id} className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-3">
                {getStatusIcon(fileItem.status)}
                <div>
                  <p className="text-sm font-medium text-gray-900">
                    {fileItem.name}
                  </p>
                  {fileItem.originalName !== fileItem.name && (
                    <p className="text-xs text-gray-500">
                      Originally: {fileItem.originalName}
                    </p>
                  )}
                </div>
              </div>
              
              <span className="text-sm text-gray-600">
                {getStatusText(fileItem.status)}
              </span>
            </div>
            
            {/* Progress Bar */}
            {(fileItem.status === 'analyzing' || fileItem.status === 'converting') && (
              <div className="mb-2">
                <div className="bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-gradient-to-r from-blue-500 to-purple-500 h-2 rounded-full transition-all duration-300"
                    style={{ width: `${fileItem.progress}%` }}
                  />
                </div>
                <p className="text-xs text-gray-500 mt-1">
                  {fileItem.progress}% complete
                </p>
              </div>
            )}
            
            {/* Error Message */}
            {fileItem.status === 'error' && fileItem.error && (
              <p className="text-sm text-red-600 mt-2">
                ❌ {fileItem.error}
              </p>
            )}
            
            {/* Download Link */}
            {fileItem.status === 'success' && fileItem.convertedUrl && (
              <a
                href={fileItem.convertedUrl}
                download={fileItem.name}
                className="inline-flex items-center text-sm text-green-600 hover:text-green-700 mt-2"
              >
                ✅ Download converted file
              </a>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}