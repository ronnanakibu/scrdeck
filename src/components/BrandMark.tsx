import React from "react";

interface BrandMarkProps {
  size?: number;
  className?: string;
}

export const BrandMark: React.FC<BrandMarkProps> = ({
  size = 24,
  className = "",
}) => {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-hidden="true"
    >
      {/* Back device outline */}
      <rect
        x="3"
        y="5"
        width="11"
        height="16"
        rx="2"
        stroke="currentColor"
        strokeWidth="1.75"
        opacity="0.55"
      />
      {/* Front device outline */}
      <rect
        x="10"
        y="3"
        width="11"
        height="16"
        rx="2"
        fill="var(--bg-surface)"
        stroke="var(--accent-lime)"
        strokeWidth="1.75"
      />
      {/* Connection bridge point */}
      <circle cx="15.5" cy="11" r="2" fill="var(--accent-lime)" />
    </svg>
  );
};
