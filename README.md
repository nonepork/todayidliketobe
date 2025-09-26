<div align="center">

Today I'd like to be

</div>

## Table of contents

- [What is this](#what-is-this)
- [Installation](#installation)
- [Usage](#preview)
- [FAQ](#faq)
  - [What is this name](#what-is-this-name)
  - [Artefacts](#artefacts)

## What is this?

This is a simple cli tool for switching your git account, to be specific:

- Generates ssh key
- Automatically configure user.name and user.email
- Automatically sets remote

## Installation

WIP

## Usage

```bash
tilb

tilb -h --help
tilb list
tilb add [name]
tilb switch [name]
tilb remove [name]
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
