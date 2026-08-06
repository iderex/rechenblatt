<!--
Four headings, and each one is read. Delete a heading only if you can say why it
does not apply; an empty heading is worth more than a deleted one, because it
shows the question was asked.
-->

## What changed

<!-- Not a restatement of the diff, which the reader already has. What is
different about the software or the tree now, and why that shape. -->

## What failure it prevents

<!-- Where this is a correction, say what was wrong and how it was found. -->

## Closes

<!-- Closes #NNN. Use the keyword so the tracker does not need a second pass.
If this closes nothing, say what issue it is part of. -->

## Evidence

<!--
Every number in this body carries the command that produced it, run at the
commit being pushed and against the reference the reader will have, not your
working tree. A number without its command is a claim, and a claim is fine as
long as it says it is one.

If this change moved a fidelity or compatibility measurement, give the before
and the after and the command that produced both. If it moved neither, say so.
-->

## Who has read this

<!-- Stated plainly either way. A body saying nobody else has read this is worth
more than one that is silent about it. -->

---

<!--
Before pushing:

    git commit -s          every commit carries a Signed-off-by matching its author
    git diff --name-only origin/main...HEAD    nothing outside the issue's Scope

The gates that judge this are in .github/workflows and are listed by
`git ls-files .github/workflows` rather than here, so this comment cannot drift
against them. `gh pr checks <number>` shows which of them actually ran.
-->
