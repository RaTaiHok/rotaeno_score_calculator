export function normalize(text) {
  return String(text ?? "")
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "");
}

export function fuzzyMatch(text, query) {
  if (!query) {
    return true;
  }

  if (text.includes(query)) {
    return true;
  }

  return isSubsequence(query, text);
}

function isSubsequence(pattern, text) {
  let p = 0;
  let t = 0;

  while (p < pattern.length && t < text.length) {
    if (pattern[p] === text[t]) {
      p += 1;
    }
    t += 1;
  }

  return p === pattern.length;
}
