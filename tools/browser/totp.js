// RFC 6238 time-based one-time passwords for the acceptance suite: HMAC-SHA1, 6 digits and
// 30-second steps, which is what riAuth enrolls by default. Uses node:crypto only.
import { createHmac } from 'node:crypto';

const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

// RFC 4648 base32, case-insensitive, with optional padding.
export function base32Decode(text) {
  const clean = text.toUpperCase().replace(/=+$/, '');
  let bits = 0, value = 0;
  const out = [];
  for (const char of clean) {
    const index = ALPHABET.indexOf(char);
    if (index < 0) throw new Error(`Invalid base32 character ${JSON.stringify(char)}`);
    value = (value << 5) | index; bits += 5;
    if (bits >= 8) { bits -= 8; out.push((value >>> bits) & 0xff); }
  }
  return Buffer.from(out);
}

export const PERIOD = 30;
export const step = (at = Date.now()) => Math.floor(at / 1000 / PERIOD);

// The code for an explicit step (RFC 4226 dynamic truncation).
export function hotp(secret, counter) {
  const message = Buffer.alloc(8);
  message.writeBigUInt64BE(BigInt(counter));
  const mac = createHmac('sha1', base32Decode(secret)).update(message).digest();
  const offset = mac[mac.length - 1] & 0x0f;
  const binary = mac.readUInt32BE(offset) & 0x7fffffff;
  return String(binary % 1_000_000).padStart(6, '0');
}

export const totp = (secret, at = Date.now()) => hotp(secret, step(at));

// Six digits that match none of the steps the server accepts right now (previous, current
// and next), so a "wrong code" test cannot pass by luck.
export function wrongCode(secret, at = Date.now()) {
  const accepted = new Set([-1, 0, 1].map((offset) => hotp(secret, step(at) + offset)));
  for (let n = 0; ; n += 1) {
    const candidate = String((Number(totp(secret, at)) + 500_000 + n) % 1_000_000).padStart(6, '0');
    if (!accepted.has(candidate)) return candidate;
  }
}
