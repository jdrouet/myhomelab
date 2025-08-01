# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/myhomelab/releases/tag/myhomelab-adapter-http-server-v0.1.0) - 2025-08-01

### Added

- create endpoint to list sensors
- add healthcheck to sensors
- create entrypoint to execute commands

### Fixed

- address clippy suggestions
- ensure range_y can be computed

### Other

- rename agent to sensor
- move manager to trait
- merge dataset with server config
- remove MetricHeader
- use MetricRef instead of Metric
- use slice in ingest
- remove period from timeseries
- make agents spawn a thread
- move adapters to a shared directory
