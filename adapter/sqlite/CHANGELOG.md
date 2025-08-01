# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/myhomelab/releases/tag/myhomelab-adapter-sqlite-v0.1.0) - 2025-08-01

### Added

- create event entity ([#11](https://github.com/jdrouet/myhomelab/pull/11))
- move config to file

### Fixed

- ensure range_y can be computed

### Other

- remove MetricFacade
- remove MetricHeader
- rename MetricRef to Metric
- use MetricRef instead of Metric
- create metric facade
- use slice in ingest
- merge metrics into single table
- remove period from timeseries
- make all timestamps u64
- move adapters to a shared directory
