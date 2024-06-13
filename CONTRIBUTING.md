# Contributing to `pigg`

We welcome your contributions to "pigg"!

You should read and agree to the contributor covent, to make sure you are able to comply with how we'd like this
community to run and behave (no matter how small it may be!).

## Ways to Contribute

Ways you can contribute are:

* Clone the repo and build from source (`make`) or `cargo install pigg` (once we have it published)
* Feedback and comments via Discussions in GitHub, or reddit or other methods when we have them setup
* Creating issues for bugs you find
* Creating issues for ideas of improvements or new features
* Creating issues (for now) with examples of how you have used it, configurations you have created and the associated
  hardware or functionality enabled via the Pi
* Commenting on existing issues for bugs, enhacements, new features roadmap, etc.
* Code contributions via a PR, that describes what the PR brings

## Raspberry Pi Expertise

We started this project, mainly as a learning exercise related to using rust, iced and rppal on Raspberry Pi hardware.

We are not Raspberry Pi hardware experts, and we still have a lot to learn about the many advanced uses of GPIO on
Raspberry Pi,
functionality such as I2C buses, SPI, UART, DPI etc.

We are particularly interested in contributions from experts in the Pi hardware and how to configure and use those
hardware functionalities,
and how to connect them to a GUI experience in interesting and useful ways.

So, if you are knowledgeable in use of specific Pi hardware features beyond simple Input/Output, please give `pigg` a
try,
be patient with us as the first releases will be very simple and limited in functionality, and help us improve it, in
any of the ways of
contributing listed above!

## Code Contribution Workflow

The code contribution workflow is just standard:

* Fork the repo
* Modify or add to the code
* Push commits (with good commit messages) to your fork
* Create a Pull Request to this repo. Reference an issue if one exists (you can use smart commit messages like "Fixes
  #123". If not describe what the PR does.
* We will review and either request some changes, or merge.

If you become a regular contributor and wish to join the team, then we can add you to this repo as a contributor and you
can work on branches of this repo and a streamlined workflow.

## Best Practices

We want to learn and improve as engineers via this project, and so we will strive to apply these best practices in our
own contributions. So it's logical that we ask other contributors to do the same (although we may be more lenient
on you :-) ).

### GH Discussions

If you just want to bounce ideas around, go ahead and share your ideas in
the [GH Discussions](https://github.com/andrewdavidmackenzie/pigg/discussions).
If a consensus is reached then an issue can easily be created.

### GH Issues

Our [GH Issues](https://github.com/andrewdavidmackenzie/pigg/issues) are fairly standard.
We have some templates for different issues types, and may add more over time. Please file issues and complete as much
of the template as applies.

Apply labels from the list of existing labels that you think apply.

Self-assign yourself if you *really* want to be the person working on it, but only if you plan to start on it fairly
soon.

When starting work on an issue, self-assign yourself, so someone else also doesn't start working on it.

### Pull Requests

If you are creating a Pull Request to fix an issue, you can use a "Smart Commit Message" in the PR description, such
as "Fixes #123". When the PR is merged, the issue will be automatically closed by it. It's good practice
to refer to an issue in a PR so people know what you are trying to achieve with it, and you don't need to
rewrite that in the PR description.

As your work on the PR progresses, commit often, each time you make an incremental change that adds something
and that compiles and works. This is helpful in separating unrelated changes, and gives you a point to fall back to
when you get into trouble. Just revert back to the previous commit point and try again.

It's often the case that you wish to push to the PR, but it's not done yet, and you don't want someone to merge
it by mistake. Use the GH feature to mark the PR as a DRAFT and it cannot be merged until you mark it ready.

When done, remove the DRAFT status, making it ready for review. Assign one or more reviewers.

If you are a reviewer, submit the review as you see fit (request changes, approved etc), and maybe add a
"LGTM" (Looks good to me)) comment to the PR, to let the author know it's good to merge for you.

If the PR is self-contained, then just squash all your commits together as you merge (GH offers this choice).

Optionally, you could (via git) selectively group and squash commits into a small number of separate commits
where each one is self-contained and makes sense.

### Code - Formatting

Please either configure your IDE to run rust `rustfmt` and "optimize imports", or run `rustfmt` before commiting
to ensure consistent code formatting, and avoiding additional changes in PRs due to pure formatting changes, not
real code changes.

### Code - Unit Tests

Please add unit tests for as much of the code as you can. We measure coverage
in [codecov.io](https://app.codecov.io/gh/andrewdavidmackenzie/pigg). Currently it is a bit low but we plan to put
in work to raise it. 70% above test coverage is a pretty good goal for new contributions.

### Code - UI testing

We still need to study how to do any UI testing with Iced. We will update these guidelines when we
learn more and decide how to go about it.
