# Terminal UI Troubleshooting Guide

This document covers common terminal UI rendering issues and their solutions.

## 1. Newline vs Word Wrap Issue

### Problem Description
When building terminal UIs, you might encounter double spacing between lines when using explicit newlines (`\n` or `Print("\n")`). This happens because:

1. Terminal automatically wraps lines when they reach the terminal width
2. Adding explicit `\n` causes an additional line break
3. Result: Double spacing that looks unprofessional

### Example of the Problem
```rust
// BAD: This causes double spacing
execute!(stdout, Print("Some text that fills the terminal width"))?;
execute!(stdout, Print("\n"))?;  // Extra newline causes double spacing!
```

### Solution
Let the terminal handle line wrapping naturally:

```rust
// GOOD: Terminal auto-wraps, no explicit newlines needed
execute!(stdout, Print("Some text that fills the terminal width"))?;
// Just continue with the next content - terminal handles wrapping
```

### Key Points
- Terminal automatically wraps when content reaches the edge
- Only use explicit newlines when you actually want blank lines
- For continuous content, rely on terminal's built-in wrapping behavior

## 2. Centering Line Fill Issue

### Problem Description
When centering text in a fixed-width terminal line, you might find the line is sometimes one character short. This happens due to integer division when calculating padding.

### Example of the Problem
```rust
// BAD: Can leave line one character short
let padding = " ".repeat((total_width - text.len()) / 2);
let centered = format!("{}{}{}", padding, text, padding);
// When (total_width - text.len()) is odd, we lose a character!
```

### Mathematical Example
- Terminal width: 80
- Margins: 2 on each side (4 total)
- Inner width: 76
- Text length: 21
- Space to fill: 76 - 21 = 55
- Padding calculation: 55 / 2 = 27 (integer division)
- Total used: 27 + 21 + 27 = 75 (missing 1 character!)

### Solution
Calculate left and right padding separately:

```rust
// GOOD: Always fills the exact width
let title_len = title.len();
let left_padding = (inner_width.saturating_sub(title_len)) / 2;
let right_padding = inner_width.saturating_sub(title_len + left_padding);

// This ensures: left_padding + title_len + right_padding = inner_width
```

### Implementation Pattern
```rust
// Full example for centering with exact width
let margin = 2;
let inner_width = terminal_width - (margin * 2);
let content = "My Title";
let content_len = content.len();

// Calculate padding
let left_pad = (inner_width - content_len) / 2;
let right_pad = inner_width - content_len - left_pad;

// Render with exact width
print!("{}", " ".repeat(margin));           // Left margin
print!("{}", " ".repeat(left_pad));         // Left padding
print!("{}", content);                      // Centered content
print!("{}", " ".repeat(right_pad));        // Right padding
print!("{}", " ".repeat(margin));           // Right margin
```

### Key Points
- Integer division can lose precision when centering
- Always calculate right padding as `total - used` rather than assuming symmetry
- This ensures your line fills exactly to the terminal width
- Prevents visual glitches when terminal is resized

## Common Debugging Tips

1. **Use visual indicators during development:**
   ```rust
   #[cfg(debug_assertions)]
   const MARGIN_CHAR: char = '.';  // Shows dots in debug builds
   #[cfg(not(debug_assertions))]
   const MARGIN_CHAR: char = ' ';  // Normal spaces in release
   ```

2. **Debug line boundaries with margin indicators:**
   ```rust
   // Debug helper for line alignment - shows dots in debug builds
   #[cfg(debug_assertions)]
   const LINE_PREFIX: char = '.';
   #[cfg(not(debug_assertions))]
   const LINE_PREFIX: char = ' ';

   // Helper method to generate margin strings
   fn margin_str(count: usize) -> String {
       LINE_PREFIX.to_string().repeat(count)
   }

   // Usage in rendering
   execute!(
       stdout,
       Print(Self::margin_str(margin)),  // Shows dots in debug mode
       Print("Your content here"),
       Print(Self::margin_str(margin)),  // Makes line boundaries visible
   )?;
   ```

   This technique helps you:
   - See exactly where each line starts and ends
   - Identify alignment issues quickly
   - Verify that lines fill the full terminal width
   - Debug spacing problems without counting characters manually

   In debug builds, you'll see something like this (with dots showing margins):
   ```
   ..                                               Agent Pool Manager                                                ..
   ..┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────┐..
   ..│ Create New Agent                                                                                              │..
   ..├───────────────────────────────────────────────────────────────────────────────────────────────────────────────┤..
   ..│ Enter agent name:                                                                                             │..
   ..│ Press Enter to continue, q to go back                                                                         │..
   ..└───────────────────────────────────────────────────────────────────────────────────────────────────────────────┘..
   ```

   The dots clearly show:
   - The margin width (4 dots = margin of 4)
   - That each line starts and ends at the same position
   - Any misalignment issues become immediately visible
   - Whether content is properly filling the available width

   In release builds, the dots are replaced with spaces, rendering a clean UI.

3. **Test with different terminal widths:**
   - Resize your terminal while the app is running
   - Test both odd and even widths
   - Ensure layouts remain consistent

4. **Count characters carefully:**
   - Remember Unicode characters may have different display widths
   - Use `unicode-width` crate if dealing with non-ASCII text
   - But be careful - sometimes simpler is better for basic ASCII UIs

## 3. Unicode Character Width Issues

### Problem Description
When using Unicode characters like arrows (▶), emojis, or special symbols, you might find your line padding calculations are off by a few characters. This happens because some Unicode characters take up more than one column width in the terminal.

### Example of the Problem
```rust
// BAD: String length doesn't equal display width
let status_text = format!(" ▶ {}", msg);  // Arrow ▶ displays as 2 columns wide
let padding = inner_width.saturating_sub(status_text.len());  // Wrong! len() counts bytes, not display width
```

### Why This Happens
- `.len()` returns the byte length of the string
- Unicode characters like "▶" might be 3 bytes but display as 2 columns
- This causes padding calculations to be off by the difference

### Solution
Use the `unicode-width` crate to get actual display width:

```rust
// GOOD: Use unicode width for accurate display calculations
use unicode_width::UnicodeWidthStr;

let status_text = format!(" ▶ {}", msg);
let status_width = UnicodeWidthStr::width(status_text.as_str());
let padding = inner_width.saturating_sub(status_width);
```

### Common Problem Characters
- Arrows: ▶ ▷ ◀ ◁ (often 2 columns wide)
- Box drawing: ╔ ╗ ╚ ╝ (can be ambiguous)
- Emojis: 😀 🚀 ⚡ (usually 2 columns wide)
- CJK characters: 中文, 日本語 (2 columns wide)

### Best Practices
1. Always use `UnicodeWidthStr::width()` for padding calculations when using Unicode
2. Test your UI with different terminal emulators (they may handle widths differently)
3. Consider using ASCII alternatives for better compatibility (e.g., ">" instead of "▶")
4. Be consistent - if you use Unicode characters, handle their width properly everywhere

## Related Issues
- Terminal color codes don't count toward display width
- Some terminals handle line wrapping differently
- Unicode box-drawing characters can have width ambiguities
- Different terminals may render the same Unicode character with different widths