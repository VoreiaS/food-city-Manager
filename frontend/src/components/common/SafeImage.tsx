import { useState, type ImgHTMLAttributes } from "react";
import { clsx } from "clsx";
import { Package, Store } from "lucide-react";

type SafeImageSrc = string | null | undefined;

interface SafeImageProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "src"> {
  src: SafeImageSrc;
  alt: string;
  fallback?: "package" | "store" | "none";
  fallbackClassName?: string;
}

export function SafeImage({
  src,
  alt,
  fallback = "package",
  className,
  fallbackClassName,
  ...rest
}: SafeImageProps) {
  const [errored, setErrored] = useState(false);

  if (!src || errored) {
    if (fallback === "none") return null;
    const Icon = fallback === "store" ? Store : Package;
    return (
      <div
        className={clsx(
          "flex items-center justify-center bg-gray-100 text-gray-400",
          className,
          fallbackClassName,
        )}
      >
        <Icon size={24} />
      </div>
    );
  }

  return (
    <img
      src={src}
      alt={alt}
      className={className}
      onError={() => setErrored(true)}
      {...rest}
    />
  );
}
