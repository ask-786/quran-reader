const EASTERN_ARABIC_DIGITS = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];

/** Convert a non-negative integer to Eastern Arabic-Indic numerals (٠-٩). */
export function toArabicNumerals(n: number): string {
  return String(n)
    .split('')
    .map((c) => (c >= '0' && c <= '9' ? EASTERN_ARABIC_DIGITS[Number(c)] : c))
    .join('');
}
