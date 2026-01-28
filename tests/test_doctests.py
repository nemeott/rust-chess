"""Simple testing suite, that tests the docstrings generated from the Rust code."""

import doctest
import textwrap
from types import ModuleType

import rust_chess


def test_rust_docstrings() -> None:
    """Run the docstring tests on rust_chess."""
    run_markdown_doctests(rust_chess)


def run_markdown_doctests(module: ModuleType) -> None:
    """Run doctests on a module, ignoring markdown code block fences (```).

    Markdown code blocks are not supported by doctest, so we need to parse them manually.
    Doctest doesn't automatically check builtins, so we have to manually check them.
    Doctest also doesn't run non-Python docstrings, so we have to fake them.
    """
    runner = doctest.DocTestRunner()
    # runner = doctest.DocTestRunner(optionflags=doctest.ELLIPSIS | doctest.NORMALIZE_WHITESPACE)

    # Let doctest use the module in its tests
    globs = {"rust_chess": module}

    # Test all classes
    for name in dir(module):
        cls = getattr(module, name)
        if not isinstance(cls, type):  # Skip non-classes
            continue

        # Test the class docstring
        run_doctest_on_object(cls, globs, runner)

        # Test all methods of the class
        for attr_name in dir(cls):
            method = getattr(cls, attr_name)

            run_doctest_on_object(method, globs, runner, cls_name=name)

    runner.summarize()
    assert runner.failures == 0


def run_doctest_on_object(
    obj,
    globs: dict[str, ModuleType],
    runner: doctest.DocTestRunner,
    cls_name: str | None = None,
) -> None:
    """Run doctest on a single object (class or method)."""
    # Check if class or method has a docstring
    doc = getattr(obj, "__doc__", None)
    if not doc:
        return

    # Remove markdown fences and TODO comments
    lines = doc.splitlines()
    filtered_lines = [
        line for line in lines if not line.strip().startswith("```") and not line.strip().startswith("TODO")
    ]
    docstring = textwrap.dedent("\n".join(filtered_lines))

    # Check if there are examples in the markdown codeblocks
    parser = doctest.DocTestParser()
    examples = parser.get_examples(docstring)
    if not examples:
        return

    # Set the test name to the class name, or class.method name
    test_name = cls_name + "." + obj.__name__ if cls_name else getattr(obj, "__name__", str(obj))
    test = doctest.DocTest(
        examples=examples,
        globs=globs,
        name=test_name,
        filename=None,
        lineno=0,
        docstring=docstring,
    )

    # Run doctest on our newly created doctest object
    runner.run(test)
