export function HexToRgba(string: string, alpha = 1) {
    if (!string) return "rgba(0,0,0,0)";
    const hex = string.replace("#", "");
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

// shade rgb
// from https://stackoverflow.com/a/13542669/11934162
export const RGBLinearShade = (color: string, percentage: number = -0.2) => {
    const i = parseInt,
        r = Math.round,
        [a, b, c, d] = color.split(","),
        lz = percentage < 0,
        t = lz ? 0 : 255 * percentage,
        P = lz ? 1 + percentage : 1 - percentage;
    return (
        "rgb" +
        (d ? "a(" : "(") +
        r(i(a[3] == "a" ? a.slice(5) : a.slice(4)) * P + t) +
        "," +
        r(i(b) * P + t) +
        "," +
        r(i(c) * P + t) +
        (d ? "," + d : ")")
    );
};
