//! Output sanitization — prevents terminal injection attacks.
//!
//! Model output can contain arbitrary bytes. A malicious or confused model
//! could emit ANSI escape sequences that manipulate the user's terminal
//! (change title, write to clipboard via OSC 52, issue commands via DCS,
//! or hide text with CSI sequences). This module strips all such sequences
//! before rendering.

/// Strip ANSI escape sequences and control characters from model output.
///
/// This prevents terminal injection attacks where a malicious model response
/// could manipulate the user's terminal.
///
/// Strips:
/// 1. CSI sequences: `\x1b[` ... (parameter bytes 0x30-0x3F, intermediate 0x20-0x2F, final 0x40-0x7E)
/// 2. OSC sequences: `\x1b]` ... ST (including OSC 8 hyperlinks)
/// 3. DCS sequences: `\x1bP` ... ST
/// 4. APC sequences: `\x1b_` ... ST
/// 5. PM sequences:  `\x1b^` ... ST
/// 6. SOS sequences: `\x1bX` ... ST
/// 7. Control chars 0x00-0x1F except `\t` (0x09), `\n` (0x0A), `\r` (0x0D)
/// 8. DEL (0x7F)
///
/// ST (String Terminator) = `\x1b\\` or `\x9c`
pub fn sanitize_for_terminal(s: &str) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        // Check for ESC (0x1B) — start of escape sequence
        if b == 0x1B {
            if i + 1 < len {
                match bytes[i + 1] {
                    // CSI: ESC [
                    b'[' => {
                        i = skip_csi(bytes, i + 2);
                        continue;
                    }
                    // OSC: ESC ]
                    // DCS: ESC P
                    // APC: ESC _
                    // PM:  ESC ^
                    // SOS: ESC X
                    b']' | b'P' | b'_' | b'^' | b'X' => {
                        i = skip_string_sequence(bytes, i + 2);
                        continue;
                    }
                    // ESC \ (ST on its own, skip both bytes)
                    b'\\' => {
                        i += 2;
                        continue;
                    }
                    // Any other ESC + byte: skip both (Fe/Fp/Fs sequences)
                    _ => {
                        i += 2;
                        continue;
                    }
                }
            } else {
                // Trailing ESC at end of string
                i += 1;
                continue;
            }
        }

        // Check for C1 control codes encoded as single bytes (0x80-0x9F)
        // 0x9B = CSI, 0x9C = ST, 0x9D = OSC, 0x90 = DCS, 0x9F = APC, 0x9E = PM, 0x98 = SOS
        // These appear in raw byte form (not valid UTF-8 on their own, but could appear
        // in mixed content). Since we iterate bytes and the input is valid UTF-8,
        // multi-byte UTF-8 sequences starting with 0x80-0x9F are actually continuation
        // bytes of valid codepoints. We handle 0x9C (ST as single byte) if somehow present.

        // Control character filtering (0x00-0x1F, 0x7F)
        if b <= 0x1F {
            match b {
                0x09 | 0x0A | 0x0D => {
                    // Tab, newline, carriage return — keep
                    out.push(b as char);
                    i += 1;
                }
                _ => {
                    // Strip all other control chars
                    i += 1;
                }
            }
            continue;
        }

        if b == 0x7F {
            // DEL — strip
            i += 1;
            continue;
        }

        // Regular character — figure out how many bytes this UTF-8 char occupies
        // and push the whole thing
        if b < 0x80 {
            out.push(b as char);
            i += 1;
        } else {
            // Multi-byte UTF-8: determine length from leading byte
            let char_len = utf8_char_len(b);
            if i + char_len <= len {
                // Safety: we know the input is valid UTF-8
                let ch = &s[i..i + char_len];
                out.push_str(ch);
            }
            i += char_len;
        }
    }

    out
}

/// Skip a CSI sequence: ESC [ <params> <intermediates> <final byte>
/// Parameter bytes: 0x30-0x3F
/// Intermediate bytes: 0x20-0x2F
/// Final byte: 0x40-0x7E
///
/// Returns the index after the sequence.
fn skip_csi(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start;

    // Skip parameter bytes (0x30-0x3F: digits, semicolons, etc.)
    while i < len && (0x30..=0x3F).contains(&bytes[i]) {
        i += 1;
    }

    // Skip intermediate bytes (0x20-0x2F)
    while i < len && (0x20..=0x2F).contains(&bytes[i]) {
        i += 1;
    }

    // Skip the final byte (0x40-0x7E)
    if i < len && (0x40..=0x7E).contains(&bytes[i]) {
        i += 1;
    }

    i
}

/// Skip an OSC/DCS/APC/PM/SOS sequence — everything up to and including ST.
/// ST = ESC \ (0x1B 0x5C) or the single byte 0x9C.
/// Also terminates on BEL (0x07), which xterm uses as an alternative ST for OSC.
///
/// Returns the index after the sequence.
fn skip_string_sequence(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start;

    while i < len {
        match bytes[i] {
            // ESC \ — two-byte ST
            0x1B if i + 1 < len && bytes[i + 1] == b'\\' => {
                return i + 2;
            }
            // Single-byte ST
            0x9C => {
                return i + 1;
            }
            // BEL — xterm-style terminator for OSC
            0x07 => {
                return i + 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Unterminated sequence — consume everything
    len
}

/// Determine the byte length of a UTF-8 character from its leading byte.
fn utf8_char_len(leading: u8) -> usize {
    if leading < 0x80 {
        1
    } else if leading < 0xE0 {
        2
    } else if leading < 0xF0 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_string_unchanged() {
        let input = "Hello, world! This is a normal string.";
        assert_eq!(sanitize_for_terminal(input), input);
    }

    #[test]
    fn empty_string() {
        assert_eq!(sanitize_for_terminal(""), "");
    }

    #[test]
    fn preserves_tabs_newlines_cr() {
        let input = "line1\n\tindented\r\nline3";
        assert_eq!(sanitize_for_terminal(input), input);
    }

    #[test]
    fn strips_csi_color_codes() {
        // Red text: ESC[31m ... ESC[0m
        let input = "\x1b[31mred text\x1b[0m normal";
        assert_eq!(sanitize_for_terminal(input), "red text normal");
    }

    #[test]
    fn strips_csi_bold_underline() {
        let input = "\x1b[1mbold\x1b[22m \x1b[4munderline\x1b[24m";
        assert_eq!(sanitize_for_terminal(input), "bold underline");
    }

    #[test]
    fn strips_csi_cursor_movement() {
        // Move cursor up 5 lines: ESC[5A
        let input = "before\x1b[5Aafter";
        assert_eq!(sanitize_for_terminal(input), "beforeafter");
    }

    #[test]
    fn strips_csi_256_color() {
        // 256-color: ESC[38;5;196m (red)
        let input = "\x1b[38;5;196mcolorful\x1b[0m";
        assert_eq!(sanitize_for_terminal(input), "colorful");
    }

    #[test]
    fn strips_csi_true_color() {
        // True color: ESC[38;2;255;0;0m
        let input = "\x1b[38;2;255;0;0mtrue color\x1b[0m";
        assert_eq!(sanitize_for_terminal(input), "true color");
    }

    #[test]
    fn strips_osc_hyperlink() {
        // OSC 8 hyperlink: ESC]8;;URL ST text ESC]8;; ST
        let input = "\x1b]8;;http://evil.com\x1b\\click here\x1b]8;;\x1b\\";
        assert_eq!(sanitize_for_terminal(input), "click here");
    }

    #[test]
    fn strips_osc_with_bel_terminator() {
        // OSC with BEL terminator (xterm-style)
        let input = "\x1b]0;Evil Title\x07normal text";
        assert_eq!(sanitize_for_terminal(input), "normal text");
    }

    #[test]
    fn strips_osc_title_set() {
        // Set terminal title: ESC]2;title ST
        let input = "\x1b]2;hacked terminal title\x1b\\safe text";
        assert_eq!(sanitize_for_terminal(input), "safe text");
    }

    #[test]
    fn strips_dcs_sequence() {
        // DCS: ESC P ... ST
        let input = "before\x1bPsome DCS payload\x1b\\after";
        assert_eq!(sanitize_for_terminal(input), "beforeafter");
    }

    #[test]
    fn strips_apc_sequence() {
        // APC: ESC _ ... ST
        let input = "before\x1b_application data\x1b\\after";
        assert_eq!(sanitize_for_terminal(input), "beforeafter");
    }

    #[test]
    fn strips_pm_sequence() {
        // PM: ESC ^ ... ST
        let input = "before\x1b^private message\x1b\\after";
        assert_eq!(sanitize_for_terminal(input), "beforeafter");
    }

    #[test]
    fn strips_sos_sequence() {
        // SOS: ESC X ... ST
        let input = "before\x1bXstart of string\x1b\\after";
        assert_eq!(sanitize_for_terminal(input), "beforeafter");
    }

    #[test]
    fn strips_control_chars() {
        // NUL, BEL, BS, etc.
        let input = "hello\x00\x01\x02\x07\x08world";
        assert_eq!(sanitize_for_terminal(input), "helloworld");
    }

    #[test]
    fn strips_del() {
        let input = "hello\x7Fworld";
        assert_eq!(sanitize_for_terminal(input), "helloworld");
    }

    #[test]
    fn mixed_content() {
        // Normal text + color codes + hyperlink + control chars + normal text
        let input = "Hello \x1b[31m\x1b]8;;http://evil.com\x1b\\red link\x1b]8;;\x1b\\\x1b[0m\x00 world";
        assert_eq!(sanitize_for_terminal(input), "Hello red link world");
    }

    #[test]
    fn unicode_preserved() {
        let input = "Hello \u{1F600} world \u{2603} snow";
        assert_eq!(sanitize_for_terminal(input), input);
    }

    #[test]
    fn unicode_with_escapes() {
        let input = "\x1b[31m\u{1F600}\x1b[0m normal \u{2603}";
        assert_eq!(sanitize_for_terminal(input), "\u{1F600} normal \u{2603}");
    }

    #[test]
    fn trailing_esc() {
        // ESC at end of string
        let input = "text\x1b";
        assert_eq!(sanitize_for_terminal(input), "text");
    }

    #[test]
    fn unterminated_osc() {
        // OSC without ST — should consume everything after the ESC ]
        let input = "\x1b]8;;http://evil.com";
        assert_eq!(sanitize_for_terminal(input), "");
    }

    #[test]
    fn multiple_csi_sequences() {
        let input = "\x1b[1;31;42mtext\x1b[0m\x1b[4mmore\x1b[0m";
        assert_eq!(sanitize_for_terminal(input), "textmore");
    }

    #[test]
    fn csi_with_private_params() {
        // Private CSI params start with ? or other 0x3C-0x3F chars
        // E.g., ESC[?1049h (alternate screen buffer)
        let input = "\x1b[?1049hHello\x1b[?1049l";
        assert_eq!(sanitize_for_terminal(input), "Hello");
    }

    #[test]
    fn real_world_ls_color_output() {
        // Simulate `ls --color=auto` output
        let input = "\x1b[0m\x1b[01;34msrc\x1b[0m  \x1b[01;34mtarget\x1b[0m  Cargo.toml";
        assert_eq!(sanitize_for_terminal(input), "src  target  Cargo.toml");
    }

    #[test]
    fn osc_52_clipboard_attack() {
        // OSC 52 clipboard manipulation: ESC]52;c;BASE64 ST
        let input = "normal\x1b]52;c;SGVsbG8gV29ybGQ=\x1b\\text";
        assert_eq!(sanitize_for_terminal(input), "normaltext");
    }

    #[test]
    fn nested_escapes_in_osc() {
        // ESC sequences within an OSC payload (before ST)
        let input = "\x1b]8;;\x1b[31mhttp://evil.com\x1b\\visible";
        // The ESC[ inside the OSC is just data until we hit ST (ESC\)
        // The first ESC\ terminates the OSC
        assert_eq!(sanitize_for_terminal(input), "visible");
    }
}
