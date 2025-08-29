import { useState, useRef } from "react";
import { Upload, File, X, CheckCircle, AlertCircle } from "lucide-react";

export default function DragAndDropFile() {
  const [isDragOver, setIsDragOver] = useState(false);
  const [files, setFiles] = useState([]);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef(null);

  const handleDragOver = (e) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e) => {
    e.preventDefault();
    setIsDragOver(false);

    const droppedFiles = Array.from(e.dataTransfer.files);
    console.log(droppedFiles);
    processFiles(droppedFiles);
  };

  const handleFileSelect = (e) => {
    const selectedFiles = Array.from(e.target.files);
    processFiles(selectedFiles);
  };

  const processFiles = (newFiles) => {
    const validFiles = newFiles.filter((file) => {
      // Accept common document formats
      const validTypes = [
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "text/plain",
        "image/jpeg",
        "image/png",
        "image/gif",
      ];

      const maxSize = 10 * 1024 * 1024; // 10MB

      return validTypes.includes(file.type) && file.size <= maxSize;
    });

    const fileObjects = validFiles.map((file) => ({
      id: Math.random().toString(36).substr(2, 9),
      file,
      name: file.name,
      size: file.size,
      type: file.type,
      status: "pending", // pending, uploading, success, error
      progress: 0,
    }));

    setFiles((prev) => [...prev, ...fileObjects]);
  };

  const removeFile = (id) => {
    setFiles((prev) => prev.filter((file) => file.id !== id));
  };

  const uploadFiles = async () => {
    setUploading(true);

    // Update all files to uploading status
    setFiles((prev) =>
      prev.map((file) =>
        file.status === "pending" ? { ...file, status: "uploading" } : file
      )
    );

    // Simulate upload process for each file
    for (const fileObj of files.filter((f) => f.status === "uploading")) {
      try {
        // Simulate upload progress
        for (let progress = 0; progress <= 100; progress += 20) {
          await new Promise((resolve) => setTimeout(resolve, 200));
          setFiles((prev) =>
            prev.map((file) =>
              file.id === fileObj.id ? { ...file, progress } : file
            )
          );
        }

        // Simulate API call
        await new Promise((resolve) => setTimeout(resolve, 500));

        setFiles((prev) =>
          prev.map((file) =>
            file.id === fileObj.id
              ? { ...file, status: "success", progress: 100 }
              : file
          )
        );
      } catch (error) {
        setFiles((prev) =>
          prev.map((file) =>
            file.id === fileObj.id ? { ...file, status: "error" } : file
          )
        );
      }
    }

    setUploading(false);
  };

  const formatFileSize = (bytes) => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const getFileIcon = (type) => {
    if (type.includes("pdf")) return "📄";
    if (type.includes("word")) return "📝";
    if (type.includes("image")) return "🖼️";
    return "📎";
  };

  return (
    <div className="max-w-2xl mx-auto p-6">
      <h2 className="text-2xl font-bold text-gray-800 mb-6">
        Upload Documents
      </h2>

      {/* Drag and Drop Zone */}
      <div
        className={`border-2 border-dashed rounded-xl p-8 text-center transition-all duration-200 ${
          isDragOver
            ? "border-blue-500 bg-blue-50 scale-105"
            : "border-gray-300 hover:border-gray-400"
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
      >
        <Upload
          className={`mx-auto mb-4 ${
            isDragOver ? "text-blue-500" : "text-gray-400"
          }`}
          size={48}
        />
        <h3 className="text-lg font-semibold text-gray-700 mb-2">
          Drop files here or click to browse
        </h3>
        <p className="text-gray-500 mb-4">
          Support for PDF, Images up to 10MB each
        </p>

        <input
          ref={fileInputRef}
          type="file"
          multiple
          className="hidden"
          onChange={handleFileSelect}
          accept=".pdf,.doc,.docx,.txt,.jpg,.jpeg,.png,.gif"
        />

        <button
          type="button"
          className="bg-blue-600 text-white px-6 py-2 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Select Files
        </button>
      </div>

      {/* File List */}
      {files.length > 0 && (
        <div className="mt-6">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-gray-800">
              Selected Files ({files.length})
            </h3>
            {files.some((f) => f.status === "pending") && (
              <button
                onClick={uploadFiles}
                disabled={uploading}
                className="bg-green-600 text-white px-4 py-2 rounded-lg hover:bg-green-700 disabled:bg-gray-400 transition-colors"
              >
                {uploading ? "Uploading..." : "Upload All"}
              </button>
            )}
          </div>

          <div className="space-y-3">
            {files.map((fileObj) => (
              <div
                key={fileObj.id}
                className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3 flex-1">
                    <span className="text-2xl">
                      {getFileIcon(fileObj.type)}
                    </span>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-gray-900 truncate">
                        {fileObj.name}
                      </p>
                      <p className="text-sm text-gray-500">
                        {formatFileSize(fileObj.size)}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center space-x-2">
                    {fileObj.status === "success" && (
                      <CheckCircle className="text-green-500" size={20} />
                    )}
                    {fileObj.status === "error" && (
                      <AlertCircle className="text-red-500" size={20} />
                    )}
                    {fileObj.status !== "uploading" && (
                      <button
                        onClick={() => removeFile(fileObj.id)}
                        className="text-gray-400 hover:text-red-500 transition-colors"
                      >
                        <X size={20} />
                      </button>
                    )}
                  </div>
                </div>

                {/* Progress Bar */}
                {fileObj.status === "uploading" && (
                  <div className="mt-3">
                    <div className="bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${fileObj.progress}%` }}
                      />
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      {fileObj.progress}% uploaded
                    </p>
                  </div>
                )}

                {/* Status Messages */}
                {fileObj.status === "success" && (
                  <p className="text-sm text-green-600 mt-2">
                    ✅ Upload successful
                  </p>
                )}
                {fileObj.status === "error" && (
                  <p className="text-sm text-red-600 mt-2">❌ Upload failed</p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Upload Summary */}
      {files.length > 0 && (
        <div className="mt-6 p-4 bg-gray-50 rounded-lg">
          <div className="flex justify-between text-sm">
            <span>Total Files: {files.length}</span>
            <span>
              Total Size:{" "}
              {formatFileSize(files.reduce((acc, f) => acc + f.size, 0))}
            </span>
          </div>
          <div className="flex justify-between text-sm mt-1">
            <span className="text-green-600">
              Uploaded: {files.filter((f) => f.status === "success").length}
            </span>
            <span className="text-red-600">
              Failed: {files.filter((f) => f.status === "error").length}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
