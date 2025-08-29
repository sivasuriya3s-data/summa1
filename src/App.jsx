import { useState } from "react";
import "./App.css";
import DragAndDropFile from "./components/DragAndDropFile";
function App() {
  const [selectedExam, setSelectedExam] = useState("upsc");

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center p-4">
      <div className="w-full bg-white rounded-2xl shadow-xl p-8 max-w-4xl">
        <div className="flex items-center">
          {/* Left side - Exam badges */}
          <div className="flex flex-col space-y-4">
            {["NEET", "JEE", "UPSC", "CAT", "GATE"].map((exam, i) => (
              <button
                onClick={() => setSelectedExam(exam.toLowerCase())}
                key={exam}
                className={`${
                  selectedExam === exam.toLowerCase() &&
                  "bg-gradient-to-r from-blue-600 to-purple-600 text-white"
                } text-gray-800 px-6 py-3 rounded-lg font-semibold text-center shadow-md hover:shadow-lg transition-all duration-200 hover:scale-105 cursor-pointer`}
              >
                {exam}
              </button>
            ))}
          </div>

          {/* Right side - Main content */}
          <div className="ml-10 flex-1">
            <h1 className="text-4xl font-bold text-gray-800 mb-4 leading-tight">
              Welcome to{" "}
              <span className="bg-gradient-to-r from-blue-600 to-purple-600 bg-clip-text text-transparent">
                getConvertedExams.io
              </span>
            </h1>
            <p className="text-xl text-gray-600 leading-relaxed">
              Your all-in-one
              <strong className="text-gray-800"> Competitive Exams </strong>
              Document Converter
            </p>

            <DragAndDropFile />
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
