import React from 'react';
import { CheckCircle, FileText, Image, Signature } from 'lucide-react';
import { examConfigs } from '../config/examConfigs';

interface ExamRequirementsProps {
  selectedExam: string;
}

export default function ExamRequirements({ selectedExam }: ExamRequirementsProps) {
  const config = examConfigs[selectedExam];
  
  if (!config) return null;

  const getIcon = (requirement: string) => {
    const lower = requirement.toLowerCase();
    if (lower.includes('photo')) return <Image className="w-4 h-4" />;
    if (lower.includes('signature')) return <Signature className="w-4 h-4" />;
    return <FileText className="w-4 h-4" />;
  };

  return (
    <div className="bg-blue-50 rounded-lg p-6 mb-6">
      <h3 className="text-lg font-semibold text-gray-800 mb-4 flex items-center">
        <CheckCircle className="w-5 h-5 mr-2 text-blue-600" />
        {config.name} Requirements
      </h3>
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {config.requirements.map((requirement, index) => (
          <div key={index} className="flex items-center space-x-3 text-sm text-gray-700">
            <div className="text-blue-600">
              {getIcon(requirement)}
            </div>
            <span>{requirement}</span>
          </div>
        ))}
      </div>
      
      <div className="mt-4 pt-4 border-t border-blue-200">
        <h4 className="font-medium text-gray-800 mb-2">Supported Formats & Sizes:</h4>
        <div className="flex flex-wrap gap-2">
          {config.formats.map((format) => (
            <span key={format} className="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">
              {format} (max {Math.round(config.maxSizes[format] / 1024)}KB)
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}