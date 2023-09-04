It's in here I'll try to breakdown the architecture for anyone who may wish to contribute in the future. It's quite simple, really.

# Architecture
The code is split between `app.rs` and `gui.rs`. `app.rs` contains the business logic of the app; it has the state, multiple utility types, and the implmentation of the app itself. `gui.rs`, on the other hand, contains the GUI logic; it implements `eframe::App` and handles updating the screen, rendering the GUI, etc.

Since `update` already has a mutable borrow of `self`, the implementation of `PathyApp` has to manually take in mutable references to whatever it will modify, since the borrow checker won't allow us a second mutable borrow of `self`.

## Rendering
The bulk of the code is really in the GUI. Every frame it checks the current cursor mode, and renders a tooltip based off of that. It then handles any mouse input, drags, etc., and also renders the overlay for the user. Since `egui` is an immediate mode GUI library(GUI is rerendered every frame), it makes updating the application state quite easy; as any change in state will immediately be rendered later in this frame or in the next one.

## The Magic
Of course, the "magic" is in the pre-processing and generation steps. The pre-processer rounds off all the measurements. Since we're calculating all the angles and distances here, we just save those (which makes our generation function trivial). Here's some notable things about the preprocessing calculations:

### The Distance
The distance is just calculated using the cosine of the angle. However, since the angle can sometimes be 90 degrees, this can occasionally bug out. Therefore, we have an additional check to prevent issues with divide by zero, as the angle calculation itself uses the slope, and 90 degree angles result in a slope of `Δy/0`.

### The Angle
The angle is calculated as the arc-tangent of the slope. This is fine, until you realize that we're actually lacking information here; slope doesn't tell us if we're turning to the *left* or to the *right*, which means we don't know how to turn the robot. Therfore, we've implemented a `ComplexAngle` type, which encodes both the angle of the line, and whether the x value is increasing or decreasing(going right or left). 

The `ComplexAngle` has it's accompanying `AngleDirection` type, which lets us know which way the angle is "facing". We can use this information along with the previous `ComplexAngle`'s information to calculate how much the robot will need to turn.

# Contributing
Contributing is pretty simple. Test the feature, make sure it works. Try to use commit names as per the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) standard, and before you push check your code with `clippy` (by running `cargo clippy -- -D warnings`).

**Do not push directly to the main branch.** Instead, create a feature branch, say `feature-add-5d-support`, and then open a PR(pull request) to merge it into main. This way, we'll be able to review any changes and CI will be able to automatically tell us if the code is ready to merge.
