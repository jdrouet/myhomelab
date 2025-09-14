# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/myhomelab/releases/tag/v0.1.0) - 2025-09-14

### Added

- *(miflora)* read the sensor
- create xiaomi-miflora collector
- add xiaomi lywsd03mmc atc collector
- create bluetooth collector
- init project

### Fixed

- apply clippy suggestions
- use sudo to install packages
- remove non alphanum chars in unit
- update miflora detection

### Other

- create release job
- avoid having multiple time the same job
- format code
- build debian package
- add jobs
- update dockerfiles to build deb file
- remove bluer by default
- create cargo-deb config
