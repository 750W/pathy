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
