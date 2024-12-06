# Devil ML

This is the machine learning base behind the Devil Arm. Bascially training needs to be ran, 
and generate a model in order for anything else to compile. This outputs into a temp directory for the
app which is then shared with the `devil-ml/model` crate. The `devil-ml/model` crate exports a directory
called `ARTIFACT_DIR` which contains where the model was generated.

## Setup

No setup beyond getting rust is needed, and running `cargo run` in training first.

