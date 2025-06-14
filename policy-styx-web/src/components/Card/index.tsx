import React, { FC, ReactNode } from 'react';
import classNames from 'classnames'; // A small utility for conditional classnames. `npm install classnames` if you don't have it.
import { CircleNotchIcon } from '@phosphor-icons/react';
import './index.less';

export interface CardProps {
    /** The content of the card */
    children?: ReactNode;
    /** The title of the card, displayed in the header */
    title?: ReactNode;
    /** Extra content to display in the header, like buttons or links */
    extra?: ReactNode;
    /** Shows a loading spinner and blurs the content */
    loading?: boolean;
    /** Optional custom class name for the card wrapper */
    className?: string;
    /** Optional inline styles for the card wrapper */
    style?: React.CSSProperties;
    /** An action to perform when the card is clicked */
    onClick?: () => void;
}

const Card: FC<CardProps> = (props) => {
    const { children, title, extra, loading, className, style, onClick } = props;

    // Combine classes conditionally
    const cardClassName = classNames('cp-card', className, {
        'cp-card-loading': loading,
        'cp-card-clickable': !!onClick,
    });

    const renderHeader = () => {
        if (!title && !extra) {
            return null;
        }
        return (
            <div className="cp-card-header">
                {title && <div className="cp-card-title">{title}</div>}
                {extra && <div className="cp-card-extra">{extra}</div>}
            </div>
        );
    };

    const renderLoader = () => {
        if (!loading) {
            return null;
        }
        return (
            <div className="cp-card-loader">
                <CircleNotchIcon size={32} className="spinner" />
            </div>
        );
    };

    return (
        <div className={cardClassName} style={style} onClick={onClick}>
            {renderLoader()}
            {renderHeader()}
            <div className="cp-card-body">{children}</div>
        </div>
    );
};

export default Card;