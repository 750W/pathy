# Pathy


This Pathy, a dead simple path maker for PROS and EZTemplate robots. To get started, create an [EZTemplate](https://ez-robotics.github.io/EZ-Template/) project, then generate some code with Pathy and paste it into an auton function.

The goal is for this to be the simplest way to get started writing an auton with EZTemplate.

Pathy runs natively and on the web, and can be deployed through Github Pages.

## Using Pathy

Start by setting the field size in the settings, then adjust the scale to your liking. After that, hit the create button to start making your path. Pathy provides a few basic tools (create, edit, delete, trim) to help you make your path. Once you've finished, hit `Preprocess`. Pathy will then try to round off all distances to the nearest inch and turns to the nearest angle. If you're satisfied with the result, hit Generate, and a codebox will appear with some code.

### Using the generated code

You'll probably want to paste the code into a function in your `autons.cpp`. It's very likely that you won't be done after this process; due to the imperfections of real life, you may need to tune and tweak some numbers to get a fully optimized path. It's also quite likely that you'll want to add extra code to control other motors and such. You can simply edit the code yourself to add those features.

### Testing locally

Make sure you are using the latest version of stable rust by running `rustup update`. Additionally, make sure to install [`trunk`](https://trunkrs.dev).

`cargo install --locked trunk`

`trunk serve`

On Linux you may need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you'll need to run:

`dnf install clang clang-devel clang-tools-extra libxkbcommon-devel pkg-config openssl-devel libxcb-devel gtk3-devel atk fontconfig-devel`

### Web Locally

You can compile your app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page.

We use [Trunk](https://trunkrs.dev/) to build for web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> `assets/sw.js` script will try to cache our app, and loads the cached version when it cannot connect to server allowing your app to work offline (like PWA).
> appending `#dev` to `index.html` will skip this caching, allowing us to load the latest builds during development.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. we already provide a workflow that auto-deploys our app to GitHub pages if you enable it.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.
