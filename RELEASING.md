# RELEASING

We use `cargo dist` to automate our release and package publishing process.

The steps to do a full release are:

- Merge all pending PRs planned for the release
- Create a new issue in GitHub based on the "release-manual-testing-template.md" and assign it to the milestone
  for the release.
- Go through that new issue for manual testing, performing the manual tests in the matrix and checking them
  off as done as you go, until all pass.
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
- Test as many of the installation methods as you can
- Publish to crates.io also using:
    - `cargo publish`
- Install from crates.io and check both run and have the correct version number
    - `cargo install pigg`
    - run `piggui` at the command line and check the version number displayed or run `piggui --version`
    - run `piglet --version` and check the correct version number is displayed
- Test `cargo binstall`
    - Uninstall the version just installed, using `cargo install` at the command line
    - Install the pre-built binary from GH Releases using `cargo binstall pigg`at the command line
    - run `piggui` and check the version number displayed or run `piggui --version`
    - run `piglet --version` and check the correct version number is displayed
  