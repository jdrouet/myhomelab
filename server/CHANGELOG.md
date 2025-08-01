# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/jdrouet/myhomelab/releases/tag/myhomelab-server-v0.1.0) - 2025-08-01

### Added

- create event entity ([#11](https://github.com/jdrouet/myhomelab/pull/11))
- create entrypoint to execute commands
- move config to file
- make interval configurable
- implement simple miflora reader
- implement from_env for agent manager
- *(client-web)* plug web client in http adapter
- create atc reader
- create simple dashboard repository based on files
- create integrated agent
- skaffold first agent system
- create ingest http endpoint
- create config builder
- create server crate

### Fixed

- disable bluetooth reader

### Other

- remove unused deps
- rename agent to sensor
- move manager to trait
- update dockerfile to cross build binaries
- format code
- rename readers
- rename traits
- rename reader to sensor
- merge dataset with server config
- remove MetricFacade
- rename MetricRef to Metric
- use MetricRef instead of Metric
- push bigger sets of metrics
- implement simple collector
- rename agent-core to agent-manager
- make agents spawn a thread
- update debian loader
- split dockerfile to handle alpine and debian
- update dockerfile
- add dockerfiles
- move adapter-http
