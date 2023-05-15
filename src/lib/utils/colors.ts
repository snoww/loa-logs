export function HexToRgba(string: string, alpha = 1) {
    if (!string) return "rgba(0,0,0,0)";
    const hex = string.replace("#", "");
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}
