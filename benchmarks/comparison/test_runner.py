"""Guard against publishing invalid programs or misleading summary statistics."""

import importlib.util
from pathlib import Path
import sys
import unittest

spec = importlib.util.spec_from_file_location("comparison_runner", Path(__file__).with_name("run.py"))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ValidationTests(unittest.TestCase):
    def test_successful_program_returns_elapsed_time(self):
        output, elapsed = runner.checked_run([sys.executable, "-c", "print(42)"], "42\n")
        self.assertEqual(output, "42\n")
        self.assertGreater(elapsed, 0)

    def test_zero_exit_with_wrong_answer_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "failed validation"):
            runner.checked_run([sys.executable, "-c", "print(41)"], "42\n")

    def test_correct_answer_with_error_exit_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "exit=1"):
            runner.checked_run([sys.executable, "-c", "print(42); raise SystemExit(1)"], "42\n")

    def test_stderr_is_rejected_even_with_correct_answer(self):
        with self.assertRaisesRegex(RuntimeError, "failed validation"):
            runner.checked_run([
                sys.executable, "-c", "import sys; print(42); print('failure', file=sys.stderr)",
            ], "42\n")

    def test_outlier_is_preserved_in_statistics(self):
        summary = runner.summarize([1, 2, 3, 4, 100000])
        self.assertEqual(summary["median_ns"], 3)
        self.assertEqual(summary["max_ns"], 100000)
        self.assertEqual(summary["samples"], 5)
        self.assertEqual(summary["mean_ns"], 20002)
        self.assertEqual((summary["p25_ns"], summary["p75_ns"]), (2, 4))


if __name__ == "__main__":
    unittest.main()
