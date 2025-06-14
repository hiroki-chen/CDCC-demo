

import React, { useState, FC, ChangeEvent, DragEvent } from 'react';
import { CheckCircleIcon, CodeIcon, DatabaseIcon, FileIcon } from '@phosphor-icons/react';
import './index.less'; // Import the component-specific styles

interface FileUploaderProps {
  title: string,
  description: string,
  onFileSelect: (file: File) => void,
  selectedFile: File | null,
}

export const FileUploader: FC<FileUploaderProps> = ({ title, description, onFileSelect, selectedFile }) => {
  const [isDragOver, setIsDragOver] = useState(false);

  const icon = title.includes('Data Owner') ? <DatabaseIcon size={32} weight="duotone" /> : < CodeIcon size={32} weight="duotone" />;

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length != 0) {
      onFileSelect(e.target.files[0]);
    }
  };

  const handleDragOver = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleDrop = (e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);

    if (e.dataTransfer.files && e.dataTransfer.files.length != 0) {
      onFileSelect(e.dataTransfer.files[0]);

      e.dataTransfer.clearData(); // Clear the data transfer object¸
    }
  };

  const triggerFileInput = () => {
    const fileInput = `file-input-${title.replace(/\s+/g, '-')}`;
    document.getElementById(fileInput)?.click();
  };

  return (
    <div className='upload-column-container'>
      <div className='upload-header'>
        {icon}
        <h3>{title}</h3>
      </div>
      <p className='upload-description'>{description}</p>

      {/* --- Conditional Rendering: Show selected file info OR the uploader box --- */}
      {selectedFile ? (
        // State 1: A file has been selected
        <div className="file-info-card">
          <FileIcon size={32} weight="duotone" />
          <div className="file-info-text">
            <strong title={selectedFile.name}>{selectedFile.name}</strong>
            <span>{(selectedFile.size / 1024).toFixed(2)} KB</span>
          </div>
          <CheckCircleIcon size={24} weight="fill" className="file-ready-icon" />
        </div>
      ) : (
        // State 2: No file is selected, show the uploader
        <div
          className={`file-uploader-box ${isDragOver ? 'drag-over' : ''}`}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={triggerFileInput}
          role="button"
          tabIndex={0}
        >
          {/* Hidden file input that we trigger programmatically */}
          <input
            type="file"
            id={`file-input-${title.replace(/\s+/g, '-')}`}
            style={{ display: 'none' }}
            onChange={handleFileChange}
          />
          <p>Drag & Drop File Here</p>
          <span>or click to browse</span>
        </div>
      )}
    </div>
  );

};

export default FileUploader;