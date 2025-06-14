import React, { FC, ReactNode, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { XIcon } from '@phosphor-icons/react';
import classNames from 'classnames';
import './index.less';

export interface ModalProps {
  /** Controls if the modal is visible or not */
  isOpen: boolean;
  /** Function to call when the user requests to close the modal (e.g., clicks overlay or close button) */
  onClose: () => void;
  /** The title displayed in the modal's header */
  title: ReactNode;
  /** The main content of the modal */
  children: ReactNode;
  /** Optional content for the footer, typically for action buttons */
  footer?: ReactNode;
  /** Optional custom class name for the modal content */
  className?: string;
}

const Modal: FC<ModalProps> = ({ isOpen, onClose, title, children, footer, className }) => {
  // Effect to handle closing the modal with the 'Escape' key
  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };
    if (isOpen) {
      document.body.style.overflow = 'hidden'; // Prevent background scrolling
      window.addEventListener('keydown', handleEsc);
    }
    return () => {
      document.body.style.overflow = 'unset';
      window.removeEventListener('keydown', handleEsc);
    };
  }, [isOpen, onClose]);

  // Use a portal to render the modal at the root of the document, avoiding z-index issues.
  if (!isOpen) {
    return null;
  }

  return createPortal(
    <div className="modal-overlay" role="dialog" aria-modal="true" onMouseDown={onClose}>
      <div
        className={classNames('modal-content', className)}
        onMouseDown={(e) => e.stopPropagation()} // Prevent clicks inside modal from closing it
      >
        <div className="modal-header">
          <h3 className="modal-title">{title}</h3>
          <button className="modal-close-button" onClick={onClose} aria-label="Close modal">
            <XIcon size={20} />
          </button>
        </div>
        <div className="modal-body">
          {children}
        </div>
        {footer && (
          <div className="modal-footer">
            {footer}
          </div>
        )}
      </div>
    </div>,
    document.body
  );
};

export default Modal;