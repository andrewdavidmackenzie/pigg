# RELEASING

We use `cargo dist` to automate our release and package publishing process.

The steps to do a full release are:

- Update the version number (e.g. "1.2.3") in `Cargo.toml` in a new branch, push it, create a PR,
  wait for test sto pass, then merge that PR into master
- checkout master, pull the latest version
- create a new tag, matching the version number in `Cargo.toml`
    - `git tag 1.2.3`
- push that tag to GitHub
    - `git push --tags`
- In [GitHub Actions](https://github.com/andrewdavidmackenzie/pigg/actions), an action will kick off to
  build and package all the binaries, create the GitHub release, upload the binaries
  as assets and package them in installers/scripts and push those to the relevant
  locations
- Check the release in [GitHub Releases](https://github.com/andrewdavidmackenzie/pigg/releases)
- Test as many of the install methods as you can
- Publish to [crates.io](https://crates.io/crates) also using:
    - `cargo publish`
- Test that last step with:
    - `cargo install pigg`
    - `piggui` and check the version number displayed
    - `piglet --version` and check the correct version number is displayed
