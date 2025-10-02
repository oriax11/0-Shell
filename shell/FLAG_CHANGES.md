# Flag Support Changes

## Summary

Per project requirements, flag support has been updated to match the original specifications:

### Commands WITH Flag Support
- **ls** - Supports `-l`, `-a`, `-F` flags ✅
- **rm** - Supports `-r`, `-R`, `-f` flags ✅

### Commands WITHOUT Flag Support
- **cp** - No flags supported (removed `-r`/`-R`) ❌
- **mkdir** - No flags supported (removed `-p`) ❌
- **echo** - No flags ❌
- **cd** - No flags ❌
- **pwd** - No flags ❌
- **cat** - No flags ❌
- **mv** - No flags ❌

---

## Changes Made

### 1. cp Command (src/commands/cp.rs)

**Removed:**
- `-r` / `-R` flag for recursive directory copying
- `copy_dir_recursive()` helper function

**Added:**
- Flag validation that shows error message
- Clear message: "cp does not support flags"

**Still Supported:**
- Multi-file copying: `cp file1 file2 file3 dest/`
- Single file copying: `cp src dest`

**Behavior with directories:**
```bash
$ cp directory newdir
cp: omitting directory 'directory'
```

**Behavior with flags:**
```bash
$ cp -r dir1 dir2
cp: invalid option -- 'r'
cp does not support flags
```

---

### 2. mkdir Command (src/commands/mkdir.rs)

**Removed:**
- `-p` flag for creating parent directories
- `create_dir_all()` functionality

**Added:**
- Flag validation that shows error message
- Clear message: "mkdir does not support flags"

**Still Supported:**
- Multiple directory creation: `mkdir dir1 dir2 dir3`
- Single directory creation: `mkdir newdir`

**Behavior when parent doesn't exist:**
```bash
$ mkdir deep/nested/dir
mkdir: deep/nested/dir: No such file or directory (os error 2)
```

**Behavior with flags:**
```bash
$ mkdir -p deep/nested/dir
mkdir: invalid option -- 'p'
mkdir does not support flags
```

---

## Testing

### cp Tests

✅ **Single file copy works:**
```bash
$ cp file1.txt file2.txt
$ cat file2.txt
test
```

✅ **Multi-file copy works:**
```bash
$ cp file1.txt file2.txt file3.txt dest/
$ ls dest/
file1.txt  file2.txt  file3.txt
```

✅ **Flag shows error:**
```bash
$ cp -r srcdir destdir
cp: invalid option -- 'r'
cp does not support flags
```

✅ **Directory omitted:**
```bash
$ cp testdir newdir
cp: omitting directory 'testdir'
```

---

### mkdir Tests

✅ **Single directory creation works:**
```bash
$ mkdir testdir
$ ls -d testdir
testdir
```

✅ **Multiple directory creation works:**
```bash
$ mkdir dir1 dir2 dir3
$ ls
dir1  dir2  dir3
```

✅ **Flag shows error:**
```bash
$ mkdir -p deep/nested/dir
mkdir: invalid option -- 'p'
mkdir does not support flags
```

✅ **Nested fails without parents:**
```bash
$ mkdir deep/nested/dir
mkdir: deep/nested/dir: No such file or directory (os error 2)
```

---

## Rationale

These changes align the implementation with the original project requirements:

1. **Project Spec**: Only `ls` and `rm` were specified to have flag support
2. **Simplicity**: Removes features that weren't requested
3. **Clear Errors**: Users attempting to use unsupported flags get helpful feedback
4. **Core Functionality**: All essential operations still work (multi-file cp, multiple mkdir, etc.)

---

## User Impact

### What Still Works
- ✅ Copying multiple files at once
- ✅ Creating multiple directories at once
- ✅ All error messages are clear and helpful

### What Changed
- ❌ Can't recursively copy directories with `cp -r`
- ❌ Can't create nested directories with `mkdir -p`

### Workarounds
If users need these features, they can:
1. Use the system's `cp` and `mkdir` commands (available through shell's external command support)
2. Manually create parent directories first before creating nested ones
3. Copy directories file-by-file if needed

---

## Documentation Updated

The following documentation files have been updated to reflect these changes:

1. **README.md** - Updated supported commands section and examples
2. **BUGFIX_SUMMARY.md** - Updated cp and mkdir sections
3. **FLAG_CHANGES.md** - This file (details the changes)

All examples using `-r` or `-p` flags have been removed or replaced with appropriate alternatives.

---

## Build & Test Status

✅ **Build Status:** Successful (0.69s)
✅ **cp Tests:** All passed
✅ **mkdir Tests:** All passed
✅ **Flag Validation:** Working correctly

---

**Last Updated:** October 2, 2025
**Changes Applied:** Removed unsupported flags from cp and mkdir
**Status:** ✅ Complete and tested
