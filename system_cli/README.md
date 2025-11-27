# system_cli

A command-line interface for retrieving system information.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

```bash
# Show all OS information (default behavior)
system_cli
system_cli --all

# Show specific OS information
system_cli --system-type        # or -t
system_cli --system-version
system_cli --bit-depth          # or -b
system_cli --architecture       # or -a

# Combine multiple flags
system_cli -t -b
```

## Examples

### Show all information

```bash
$ system_cli --all
OS information:
  Type: Windows
  Version: 10.0.19045
  Edition: Professional
  Bitness: 64-bit
  Architecture: x86_64
```

### Show specific fields

```bash
$ system_cli --system-type
OS type: Windows

$ system_cli -t -b
OS type: Windows
OS bitness: 64-bit
```

## Options

- `--all`: Show all OS information (default if no flags specified)
- `-t, --system-type`: Show OS type
- `--system-version`: Show OS version
- `-b, --bit-depth`: Show OS bitness (32-bit or 64-bit)
- `-a, --architecture`: Show CPU architecture

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
