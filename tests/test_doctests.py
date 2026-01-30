"""Simple testing suite, that tests the docstrings generated from the Rust code.

Usage:
    Run `pytest` from the main directory.
"""

import ast
import doctest
import textwrap
from pathlib import Path
from types import ModuleType

import rust_chess


def test_rust_docstrings() -> None:
    """Run the docstring tests on rust-chess using the .pyi stub file."""
    stub = Path(__file__).resolve().parents[1] / "rust_chess.pyi"  # ../rust_chess.pyi
    docs = collect_stub_docstrings(stub)
    run_stub_doctests(rust_chess, docs)


def collect_stub_docstrings(stub_path: Path) -> dict[str, str]:
    """Parse the stub file and collect docstrings for classes, methods, properties, and staticmethods."""
    tree = ast.parse(stub_path.read_text())
    docs: dict[str, str] = {}

    for node in tree.body:
        if not isinstance(node, ast.ClassDef):
            continue

        # Class docstring
        class_doc = ast.get_docstring(node)
        if class_doc:
            docs[node.name] = class_doc

        for item in node.body:
            if not isinstance(item, ast.FunctionDef):
                continue

            func_doc = ast.get_docstring(item)
            if not func_doc:
                continue

            # Qualname for doctest
            qualname = f"{node.name}.{item.name}"

            # Add docstring
            docs[qualname] = func_doc

    return docs


def run_stub_doctests(module: ModuleType, docs: dict[str, str]) -> None:
    """Run doctests from a stub docstring mapping against the real module."""
    # Accept ellipsis to ignore some results and normalize whitespace to ignore extra newlines expected
    runner = doctest.DocTestRunner(
        optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE | doctest.IGNORE_EXCEPTION_DETAIL
    )

    # Let doctest use the module in its tests
    globs = {"rust_chess": module}

    # Test on all docs
    for qualname, doc in docs.items():
        run_doctest_on_doc(qualname, doc, globs, runner)

    runner.summarize()
    assert runner.failures == 0


def run_doctest_on_doc(
    qualname: str,
    doc: str,
    globs: dict[str, ModuleType],
    runner: doctest.DocTestRunner,
) -> None:
    """Run doctest on a single docstring."""
    # Remove markdown fences and TODO lines
    lines = doc.splitlines()
    filtered_lines = [line for line in lines if not line.strip().startswith(("```", "TODO", "#"))]
    docstring = textwrap.dedent("\n".join(filtered_lines))

    # Check if there are examples in the markdown codeblocks
    parser = doctest.DocTestParser()
    examples = parser.get_examples(docstring)
    if not examples:
        return

    # print(qualname)  # Uncomment to check which doctests are being run

    # Set the test name to the class name, or class.method name
    test = doctest.DocTest(
        examples=examples,
        globs=globs,
        name=qualname,
        filename=None,
        lineno=0,
        docstring=docstring,
    )

    # Run doctest on our newly created doctest object
    runner.run(test)
