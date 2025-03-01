Pathy 2 is a complete rewrite of Pathy, now supporting Wolflib, our homemade PROS library, featuring rich integration on both sides.

There are a few pieces to the code: in the interest of keeping Pathy maintainable, I'll explain them below.

# Architecture
The EGUI library used by Pathy essentially renders the UI like a game: frame-by-frame. This makes writing the UI a lot easier.
Here are some of the important things to keep in mind.

## Selections
The currently hovered/selected point is stored under an `Rc<RefCell<Point>>`.
This is because Rust has limits on how many mutable references can be made to an object at once (you can only have _one_).
By using an `Rc<RefCell<Point>>`, we can have multiple mutable references.
**However, if you try use a mutable reference while another reference exists, the program will crash.**
Just be careful when working with the selected point.

## Locking
Oftentimes, the user will move their cursor faster than the app updates.
This can cause the cursor to no longer hover on a control point, despite still moving it.
If we didn't handle this, this would prevent the user from moving a point unless the moved it _extremely_ slowly.
Thus, once the user presses on a point, that point is "locked" until the user releases the mouse button.
There are two locks: one in the GUI, and one in the point itself.
The GUI lock ensures we continue handling dragging events for the locked point.
Meanwhile, the point lock ensures that the point remains in its hover state while locked.
This ensures that it both appears hovered and that the reciprocal point is updated smoothly.
