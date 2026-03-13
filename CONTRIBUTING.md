## Ways to Contribute

You can contribute in many ways, including:

- fixing bugs
- improving performance
- creating visualizers
- improving UI layout
- suggesting new features
- improving code readability
- reporting issues

Even small improvements are appreciated.


## Development Focus

ReAmped is built with several main areas of development.


### Audio Core

The audio core is responsible for:

- playback
- plugin processing
- audio analysis
- feeding visualizers

Important rules:

- avoid allocations inside the audio processing thread
- keep audio processing deterministic
- prefer simple and predictable DSP code


### Visualizers

Visualizers receive analyzed audio data and render graphics in real time.


### User Interface

The UI is built using egui.

Contributions here may include:

- layout improvements
- new UI components
- improved responsiveness
- visual polish
- customizable themes
- better meter displays


## Pull Request Guidelines

When submitting a pull request:

- keep changes focused and minimal
- avoid mixing unrelated changes
- ensure the project builds successfully
- explain what the change does and why

Small and well-scoped pull requests are easier to review.


## Code Style

General guidelines:

- keep modules simple and readable
- avoid unnecessary abstractions
- prefer explicit code over clever code
- keep audio processing fast and predictable


## Feature Requests

If you want to suggest a feature:

1. open an issue
2. describe the idea clearly
3. explain the use case

Discussion is always welcome.

