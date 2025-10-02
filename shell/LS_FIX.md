# ls -a Fix: Missing . and .. Directories

## Issue
The `ls -a` command was not displaying `.` (current directory) and `..` (parent directory) entries.

## Root Cause
The code was using `PathBuf::ends_with()` method to detect paths ending with `/.` and `/..`, but this method checks for path components, not string suffixes.

### Example:
```rust
let path = PathBuf::from("/tmp/test/.");
path.ends_with("/.")  // Returns false (wrong!)
path.to_string_lossy().ends_with("/.")  // Returns true (correct!)
```

## Files Modified
- `src/commands/ls.rs` - Lines 119-123, 142-146, 223-227

## Changes Made

### Before:
```rust
if entry.ends_with("/.") {  // PathBuf method - doesn't work!
    name = ".".to_string();
}
```

### After:
```rust
let entry_str = entry.to_string_lossy();
if entry_str.ends_with("/.") {  // String method - works correctly!
    name = ".".to_string();
}
```

## Test Results

### Before Fix:
```bash
$ ls -a
.hidden
file1
file2
```

### After Fix:
```bash
$ ls -a
.
..
.hidden
file1
file2
```

### Bash Comparison:
```bash
$ bash -c "ls -a"
.
..
file1
.hidden
file2
```

Both now correctly show `.` and `..` at the top.

## Note on Sorting
There's a minor difference in sorting between our implementation and bash:
- **Our shell**: Lexicographic sorting (`.hidden` comes before `file1`)
- **Bash**: Locale-aware, case-insensitive sorting (`.hidden` may come after regular files)

Both orderings are valid and correct. The key requirement (showing `.` and `..`) is now met.

## Verification

```bash
# Test basic ls -a
echo "ls -a" | ./target/release/0-shell
# Output: . .. .hidden file1 file2

# Test ls -la (long format)
echo "ls -la" | ./target/release/0-shell
# Output shows:
# drwxr-xr-x  2 user group 4096 Oct  2 14:26 .
# drwxr-xr-x 11 user group 4096 Oct  2 14:26 ..
# -rw-r--r--  1 user group    0 Oct  2 14:26 .hidden
# -rw-r--r--  1 user group    0 Oct  2 14:26 file1
# -rw-r--r--  1 user group    0 Oct  2 14:26 file2
```

## Status
✅ **FIXED** - `.` and `..` now appear correctly in all `ls -a` and `ls -la` outputs
