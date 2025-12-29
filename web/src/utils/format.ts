import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatBytes(bytes: number, decimals = 2) {
    if (!+bytes) return '0 B'

    const k = 1024
    const dm = decimals < 0 ? 0 : decimals
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']

    const i = Math.floor(Math.log(bytes) / Math.log(k))

    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
}

export function formatSpeed(bytesPerSec: number) {
    return `${formatBytes(bytesPerSec)}/s`
}

export function truncateMiddle(text: string, maxLength: number) {
    if (text.length <= maxLength) return text;
    const side = Math.floor((maxLength - 3) / 2);
    return text.slice(0, side) + '...' + text.slice(text.length - side);
}
