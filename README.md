<h2 align="center"><em><code>Pathy</code></em></h2>

Welcome to 750W's codebase for Pathy, a dead-simple path making tool built for WolfLib.

Pathy is built to run on the web, and is deployed on Github Pages. The current latest live version can be accessed [here](https://750w.github.io/pathy).

## Using Pathy

The field size for a standard Vex field is already set up for you.
You'll need to get a picture of the current season's field, and drop it onto Pathy to set the background.

Afterwards, simply use the mouse to draw Bezier paths on the field, then press Generate to generate the path code.

### Testing locally

To test Pathy locally, you'll need to clone the repository and run it on your local machine.

We use [Trunk](https://trunkrs.dev/) to build for the web target.
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html#dev` in a browser. See the warning below.

> Chrome likes to try to cache our app, preventing updates from showing in the browser.
> If updates still are not showing, inspect the webpage with `ctrl+shift+c`, go to the `Network` tab, and check `Disable cache`.
> Make sure to leave this DevTools window open, as otherwise the cache will be re-enabled.

### Web Deploy
1. Just run `trunk build --release`.
2. It will generate a `dist` directory as a "static html" website
3. Upload the `dist` directory to any of the numerous free hosting websites including [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
4. We have already set up a `gh-pages` workflow in the repository to automatically deploy the website to GitHub Pages on every push to the `main` branch.
> To enable Github Pages, you need to go to Repository -> Settings -> Pages -> Source -> set to `gh-pages` branch and `/` (root).
>
> If `gh-pages` is not available in `Source`, just create and push a branch called `gh-pages` and it should be available.

`a proudly made 750w tool`
