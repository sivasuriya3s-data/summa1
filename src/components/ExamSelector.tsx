import React from 'react';
import { examConfigs } from '../config/examConfigs';

interface ExamSelectorProps {
  selectedExam: string;
  onExamChange: (exam: string) => void;
}

export default function ExamSelector({ selectedExam, onExamChange }: ExamSelectorProps) {
  return (
    <div className="flex flex-col space-y-4">
      <h3 className="text-lg font-semibold text-gray-800 mb-2">Select Exam</h3>
      {Object.entries(examConfigs).map(([key, config]) => (
        <button
          key={key}
          onClick={() => onExamChange(key)}
          className={`${
            selectedExam === key
              ? "bg-gradient-to-r from-blue-600 to-purple-600 text-white shadow-lg"
              : "bg-white text-gray-800 border border-gray-200 hover:border-blue-300"
          } px-6 py-3 rounded-lg font-semibold text-center shadow-md hover:shadow-lg transition-all duration-200 hover:scale-105 cursor-pointer`}
        >
          {config.name}
        </button>
      ))}
    </div>
  );
}