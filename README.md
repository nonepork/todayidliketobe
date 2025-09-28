<div align="center">

Today I'd like to be<br><br>
<i><sup>i should probably add a preview here someday</sup></i>

</div>

## Table of contents

- [What is this](#what-is-this)
- [Installation](#installation)
- [Usage](#usage)
- [FAQ](#faq)
  - [What is this name](#what-is-this-name)
  - [Artefacts](#artefacts)

## What is this?

This is a simple cli tool for switching your git account, to be specific:

- Generates ssh key
- Automatically configure user.name and user.email
- Automatically sets remote

## Installation

Windows

```bash
cargo install todayidliketobe
```

Linux/macOS

Same as above, haven't tested but should work in theory.

## Usage

To check all possible actions:

```bash
tilb -h --help
```

Say you have two git accounts, one for work named as workuser and the other for personal named as personaluser.
You can add those users like so:

```bash
tilb add workuser workusermail@lovely.com
tilb add personaluser personaluser@lonely.com
```

Once added, you can check it has been successfully add via:

```bash
tilb list
```

Now say you already logged in as personaluser before, but you got a repo folder that requires workuser, you can cd into the folder then:

```bash
tilb switch workuser
```

then do your git commands.

Now say you got laid off, you can do

```bash
tilb remove workuser
```

## FAQ

### What is this name?

i thought it was funny the first 5 seconds i came up with this name :p

### Artefacts

This cli generates:

- ~/.tilb/config.json
- ~/.ssh/tilb/(private keys)

Which will not be deleted if uninstalled

And modifies:

- ~/.ssh/config

With banner starts as #tilb_generated
