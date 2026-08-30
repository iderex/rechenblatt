# The document features the corpus measures

A corpus assembled by collecting interesting documents measures whatever those
documents happened to contain. This file is the other way round: it enumerates
the document features first, and the corpus is then built to cover them. A
passing run against this list means the features passed. A passing run against a
directory of documents means the documents passed, which is a different sentence
and a much weaker one.

It is also the plan's spine. Every row names the issue that delivers it, so the
rendering milestones and this list point at each other and a feature nobody has
claimed is a hole in the plan rather than a hole in the corpus. The pointing is
one-way today: the rows name the issues and the issues do not yet name the rows,
which is a thing to fix in each issue rather than here. The last section is where
the holes are, by name.

## How to read a row

Each feature is one row in one of the tables below, and each row has four cells.

The identifier is what everything else points at: a corpus document's manifest
entry, a per-feature line in a fidelity report, a test name. It is lower case,
dot-separated, and each segment is letters, digits and hyphens. It is stable: a
feature is renamed by leaving the identifier alone, because a report compared
against last month's report is compared by identifier.

What it is is one line, and it says what a document does rather than what this
project does about it. A feature is a thing a document either uses or does not.

The band is where the feature sits in the priority order, and the bands are
declared in the next section. A row carrying a band that section does not declare
is refused.

Delivered by is the issue that implements the feature, written as a tracker
reference, or the word none. A row saying none belongs in the last section and
nowhere else, and a row anywhere else may not say it. That is what keeps a hole
in the plan visible instead of letting it be absent.

`crates/cli/tests/fidelity_features.rs` reads this file and refuses each of those
mistakes. It reads the bands out of the section below rather than carrying its own
copy, so a band added here is a band the checker knows about with no second edit.
What it does not read is the tracker: an issue reference is checked for its shape
and not for its existence, so a row pointing at a closed or a wrong issue is a
claim this file makes and no run here contradicts.

## The bands

The bands are a priority order over the features, and priority here means the
order in which a missing feature makes a rendering useless to somebody. They are
an argument, and the argument is written out so it can be disagreed with.

### Band 1: in nearly every document

Text, numbers, the grid, and the way a cell is filled and bordered. A document
without these is not a spreadsheet. A renderer missing one of them is not wrong
about a corner case, it is wrong about every page it produces, and no other
feature is worth measuring until band 1 is right.

### Band 2: in the documents this project is aimed at

Conditional formatting, the calculation the page needs in order to show what it
shows, page setup, and charts. The claim behind this band is that a spreadsheet
produced by an office rather than by a person keeping a list uses most of them,
and that a report, a budget or a staffing plan is very likely to use all four. It
is a claim rather than a measurement: nothing here has counted documents, issue
#26 is where the corpus that would count them is built, and issue #33 is where
the count would be published. A band that the corpus contradicts is a band to
move, and moving one costs nothing because nothing in the plan is sequenced from
the band alone.

### Band 3: present, and survivable while it is missing

Drawings other than charts, the parts of the format that appear in a minority of
documents, and the behaviours that only show up in documents built by somebody
who knew the format well. A renderer that reports these as unrepresented rather
than drawing them is honest and still useful. One that draws them wrongly is
worse than one that does not draw them at all, which is why the reporting
obligation in issue #64 sits beside them rather than after them.

## Text and cell content

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `text.shared-table` | Cell text held in the workbook's shared string table rather than in the cell | 1 | #21 |
| `text.inline` | Cell text held inline in the cell instead of in the shared table | 1 | #21 |
| `text.runs` | Formatting that changes partway through one cell's text | 1 | #21 |
| `text.phonetic` | Phonetic guides and the East Asian text properties beside them | 3 | #21 |
| `text.alignment-horizontal` | Left, centre, right, fill, justified and distributed alignment | 1 | #37 |
| `text.alignment-vertical` | Top, middle, bottom, justified and distributed vertical alignment | 1 | #37 |
| `text.centre-across-selection` | Text centred across a run of cells that were never merged | 2 | #37 |
| `text.wrap` | Text wrapped inside the cell rather than spilling out of it | 1 | #37 |
| `text.indent` | An indent applied inside the alignment of the cell | 2 | #37 |
| `text.rotation` | Text rotated by an angle, including the stacked vertical form | 2 | #37 |
| `text.shrink-to-fit` | A font size reduced until the text fits the cell | 2 | #37 |
| `text.direction-rtl` | Right-to-left text, mixed-direction text and a right-to-left sheet | 2 | #37 |
| `text.shaping` | Script-dependent glyph joining, ligatures and mark placement | 2 | #37 |

## Number formats

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `format.builtin-by-number` | A format identified by its number rather than by a format string | 1 | #19 |
| `format.sections` | The positive, negative, zero and text sections of a format string | 1 | #19 |
| `format.condition` | A section guarded by a condition on the value | 2 | #19 |
| `format.colour` | A colour named inside a format section | 2 | #19 |
| `format.literal-text` | Literal characters carried through a format, quoted or escaped | 1 | #19 |
| `format.fill-and-repeat` | The fill directive and the width-skip directive | 3 | #19 |
| `format.fraction` | A value shown as a fraction with a declared denominator form | 3 | #19 |
| `format.scientific` | Scientific notation with its exponent form | 2 | #19 |
| `format.elapsed-time` | A duration that runs past twenty-four hours instead of wrapping | 2 | #19 |
| `format.date-epoch` | Both epoch bases a workbook can declare, and the leap-year bug in the older one | 2 | #19 |
| `format.locale-separators` | Separators, month names and calendar taken from the document rather than the host | 1 | #19 |

## The grid

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `grid.column-width` | Column widths, including the default and the per-column override | 1 | #36 |
| `grid.row-height` | Row heights, including the ones a document sets rather than derives | 1 | #36 |
| `grid.hidden-row` | A row hidden, and everything positioned around it | 1 | #36 |
| `grid.hidden-column` | A column hidden, and everything positioned around it | 1 | #36 |
| `grid.merged-range` | Merged cells and what happens to the content of the cells inside them | 1 | #36 |
| `grid.frozen-panes` | Frozen rows and columns | 2 | #36 |
| `grid.split-panes` | A split view, which is not the same thing as a frozen one | 3 | #36 |
| `grid.sheet-visibility` | A sheet hidden, and the very hidden state beside it | 2 | #17 |
| `grid.overflow` | Content that does not fit its cell, spilling, clipping or refusing | 1 | #40 |

## Fills, borders and colour

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `fill.solid` | A solid cell fill | 1 | #39 |
| `fill.pattern` | A patterned fill with its foreground and background colours | 2 | #39 |
| `fill.gradient` | A gradient fill in the forms a document can declare | 3 | #39 |
| `border.styles` | Every border style and weight on the four sides | 1 | #39 |
| `border.diagonal` | The diagonal borders, which no other feature implies | 3 | #39 |
| `border.adjacent-precedence` | Which of two neighbouring cells has its border drawn between them | 1 | #39 |
| `border.gridlines` | The gridlines of the sheet itself, on screen and in print | 1 | #39 |
| `style.indirection` | The appearance of a cell resolved through the indirections of the style table | 1 | #17 |
| `theme.palette` | Colours named through the workbook theme rather than given directly | 1 | #20 |
| `theme.fonts` | The major and minor fonts a theme names, and what a document does with them | 1 | #20 |
| `theme.tint` | A theme colour modified by a tint or a shade | 2 | #20 |

## Page setup

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `page.paper-and-orientation` | Paper size and orientation, metric and imperial | 2 | #41 |
| `page.margins` | Page margins, including the header and footer margins | 2 | #41 |
| `page.header-footer-fields` | Header and footer field codes and the sections they split into | 2 | #41 |
| `page.scale` | An explicit print scale | 2 | #41 |
| `page.fit-to-pages` | A fit-to-pages instruction and the scale it produces | 2 | #41 |
| `page.manual-break` | Manual page breaks, against the automatic ones | 2 | #41 |
| `page.print-area` | A print area that is not the used range | 2 | #41 |
| `page.repeated-rows-and-columns` | Rows and columns repeated on every page | 2 | #41 |
| `page.order` | The order pages are produced in for a sheet wider and taller than one page | 3 | #41 |
| `page.print-headings` | Row and column headings printed or not | 3 | #41 |

## Conditional formatting

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `cf.rule-cell-value` | A rule comparing the value of the cell against one or two thresholds | 2 | #53 |
| `cf.rule-formula` | A rule that is a formula, evaluated relative to each cell in its range | 2 | #53 |
| `cf.rule-text` | The text rules: contains, does not contain, begins with, ends with | 2 | #52 |
| `cf.rule-blank-and-error` | The blank, non-blank, error and non-error rules | 2 | #52 |
| `cf.rule-duplicate-unique` | The duplicate-value and unique-value rules | 2 | #52 |
| `cf.rule-top-bottom` | Top and bottom by count or by percent, and how ties are treated | 2 | #53 |
| `cf.rule-average` | Above and below average, including the deviation forms | 2 | #53 |
| `cf.rule-date-period` | The date-period rules, against a clock the caller supplies | 2 | #53 |
| `cf.priority-order` | The order rules are applied in when several match one cell | 2 | #54 |
| `cf.stop-if-true` | A rule that stops the ones below it from being considered | 2 | #54 |
| `cf.overlapping-ranges` | Two rule ranges covering one cell | 2 | #54 |
| `cf.differential-format` | A format that changes the properties it names and leaves the rest | 2 | #55 |
| `cf.data-bar` | Data bars, with their axis, their direction and their negative form | 2 | #56 |
| `cf.colour-scale` | Two-colour and three-colour scales | 2 | #56 |
| `cf.icon-set` | Icon sets, including the reversed and the partly hidden forms | 2 | #56 |

## Calculation, as far as the page shows it

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `calc.reference-forms` | Every reference form a formula can carry, including the rare ones | 2 | #46 |
| `calc.nested-conditional` | Conditionals nested to the depth the format permits | 2 | #49 |
| `calc.multi-branch-conditional` | The multi-branch conditional form | 2 | #49 |
| `calc.error-trapping` | The error-trapping conditional forms | 2 | #49 |
| `calc.criteria-matching` | The matching language the conditional aggregates take, wildcards included | 2 | #49 |
| `calc.error-values` | The error values, how they propagate and how they are shown | 2 | #50 |
| `calc.circular-reference` | A circular reference, and what the document asks be done about it | 3 | #48 |
| `calc.spill` | A formula whose result occupies more than its own cell | 3 | #47 |
| `calc.cached-value-agreement` | The value the document already carries for each formula cell | 2 | #51 |

## Charts and drawings

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `chart.series-references` | A chart series pointing at ranges in a sheet | 2 | #59 |
| `chart.cached-values` | The cached values of a chart, and their disagreement with the ranges | 2 | #59 |
| `chart.plot-area` | The plot area, its frame and its position in the chart | 2 | #60 |
| `chart.axes` | Axes with their scales, ticks and number formats | 2 | #60 |
| `chart.types` | The chart types drawn, in their plain, stacked and percentage variants | 2 | #61 |
| `chart.gaps-in-data` | The ways a document can ask for a gap in a series to be treated | 3 | #61 |
| `chart.titles-and-legends` | Chart and axis titles, and the legend with its placement | 2 | #62 |
| `chart.data-labels` | Data labels with their content, position and number format | 2 | #62 |
| `chart.theme-colour` | A chart with no explicit colours, taking them from the theme and the style | 2 | #61 |
| `drawing.anchor-move-and-size` | An object that moves and sizes with the cells under it | 3 | #63 |
| `drawing.anchor-move-only` | An object that moves with the cells and keeps its size | 3 | #63 |
| `drawing.anchor-fixed` | An object that neither moves nor sizes | 3 | #63 |
| `drawing.image` | An embedded image with its cropping, rotation and transparency | 3 | #63 |
| `drawing.shape-and-textbox` | A shape or a text box with its geometry, fill, outline and text | 3 | #63 |
| `drawing.stacking-order` | The order objects are drawn in when they overlap | 3 | #63 |
| `drawing.not-printed` | An object present on screen and excluded from the print output | 3 | #63 |
| `drawing.comment` | A comment or note, which hangs off the same drawing mechanism | 3 | #63 |
| `drawing.form-control` | A form control, drawn as the document describes it | 3 | #63 |

`chart.types` is one row where every other feature here is one behaviour, and
that is deliberate rather than an oversight. Which chart types are drawn at all
is the decision issue #58 takes, and splitting the row into a type per row now
would enumerate an answer nobody has given. It splits when that record names the
types, and the row is the marker for where the split goes.

## Parts a document carries that nothing draws

These are read into the model because a later consumer needs them, and because a
part the model does not hold is a part nobody can report as unrepresented. They
are features of a document all the same, so a corpus document carrying one
declares it.

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `model.data-validation` | Data validation rules, including the list form and its source | 3 | #17 |
| `model.defined-names` | Defined names, sheet-scoped and workbook-scoped | 2 | #17 |
| `model.drawing-anchors` | The anchors of every drawing, held whether or not anything draws them | 3 | #17 |
| `model.unrepresented` | What the reader met and could not represent, recorded rather than dropped | 1 | #18 |

## Features no issue delivers

This is the hole in the plan, and it is a list rather than a silence. A feature
here is one a real document carries and no issue in this tracker claims. Adding
an issue for one of these means moving its row into a table above with the
reference filled in; leaving it here means the first release will meet the
feature and have nothing to say about it beyond that it was unrepresented.

| Id | What it is | Band | Delivered by |
| --- | --- | --- | --- |
| `table.structured-range` | A structured table over a range, with its header, totals and banding | 2 | none |
| `table.style` | A table style, which resolves underneath a conditional format and a cell style | 2 | none |
| `pivot.table` | A pivot table, with its layout, its subtotals and its own formatting | 2 | none |
| `sparkline.group` | Sparklines, which are charts drawn inside a cell rather than over the sheet | 3 | none |
| `slicer.control` | A slicer, which is a control bound to a table or to a pivot table | 3 | none |
| `hyperlink.cell` | A hyperlink on a cell, which changes how the cell is drawn as well as what it does | 3 | none |
| `sheet.background-image` | A background image behind a sheet, which is not the fill colour of the sheet | 3 | none |

## What this list is not

It is not the corpus. Issue #27 is where a corpus document declares which of these
identifiers it exercises, and where a feature with no document becomes visible.
Until then this file says what would be measured and nothing has been measured.

It is not a claim that a feature named here is implemented. Every row points at an
issue that is open or at nothing at all, and the references are shape-checked
rather than resolved, so a row is a plan and not a report.

It is not complete, and the section above is the part of that incompleteness this
file can see. A feature nobody has thought of is in neither list, and what would
find one is a corpus of real documents rather than a longer reading of the format.
