#!/usr/bin/env python3
"""
compare_images.py — Image comparison engine for visual regression testing.

Computes SSIM (Structural Similarity Index) between a captured screenshot
and a reference image. Generates a red-highlighted diff image for debugging.

Usage:
    compare_images.py <captured> <reference> [--threshold 0.85] [--diff-output diff.png]

Dependencies: Pillow, numpy (installed automatically by run_visual_tests.sh)
"""

import argparse
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def load_and_resize(captured_path: str, reference_path: str) -> tuple:
    """Load both images, resize captured to match reference dimensions."""
    ref_img = Image.open(reference_path).convert("RGB")
    cap_img = Image.open(captured_path).convert("RGB")

    if cap_img.size != ref_img.size:
        cap_img = cap_img.resize(ref_img.size, Image.LANCZOS)

    return np.array(cap_img, dtype=np.float64), np.array(ref_img, dtype=np.float64)


def compute_ssim(img1: np.ndarray, img2: np.ndarray, window_size: int = 11) -> float:
    """
    Compute SSIM between two images (mean over all channels).

    Uses the standard SSIM formula with default constants:
        C1 = (0.01 * 255)^2, C2 = (0.03 * 255)^2

    A simple uniform window (box filter) is used instead of Gaussian
    to avoid a scipy dependency while still producing reliable results.
    """
    C1 = (0.01 * 255) ** 2
    C2 = (0.03 * 255) ** 2

    ssim_channels = []
    for ch in range(img1.shape[2]):
        ch1 = img1[:, :, ch]
        ch2 = img2[:, :, ch]

        # Compute local means and variances using uniform window (box filter)
        mu1 = _uniform_filter(ch1, window_size)
        mu2 = _uniform_filter(ch2, window_size)

        mu1_sq = mu1 * mu1
        mu2_sq = mu2 * mu2
        mu1_mu2 = mu1 * mu2

        sigma1_sq = _uniform_filter(ch1 * ch1, window_size) - mu1_sq
        sigma2_sq = _uniform_filter(ch2 * ch2, window_size) - mu2_sq
        sigma12 = _uniform_filter(ch1 * ch2, window_size) - mu1_mu2

        # Clamp negative variances (numerical precision)
        sigma1_sq = np.maximum(sigma1_sq, 0)
        sigma2_sq = np.maximum(sigma2_sq, 0)

        numerator = (2 * mu1_mu2 + C1) * (2 * sigma12 + C2)
        denominator = (mu1_sq + mu2_sq + C1) * (sigma1_sq + sigma2_sq + C2)

        ssim_map = numerator / denominator
        ssim_channels.append(np.mean(ssim_map))

    return float(np.mean(ssim_channels))


def _uniform_filter(img: np.ndarray, size: int) -> np.ndarray:
    """Apply uniform (box) filter using cumulative sum for speed."""
    pad = size // 2
    padded = np.pad(img, pad, mode="reflect")
    # Cumulative sum along rows
    cs = np.cumsum(padded, axis=0)
    cs = cs[size:] - cs[:-size]
    # Cumulative sum along columns
    cs = np.cumsum(cs, axis=1)
    cs = cs[:, size:] - cs[:, :-size]
    return cs / (size * size)


def generate_diff_image(img1: np.ndarray, img2: np.ndarray, output_path: str) -> None:
    """Generate a diff image with differences highlighted in red."""
    # Compute per-pixel absolute difference
    diff = np.abs(img1 - img2)
    # Mean difference across channels
    diff_gray = np.mean(diff, axis=2)

    # Threshold: pixels with >20 difference are considered changed
    mask = diff_gray > 20

    # Create output: original image with red overlay on changed pixels
    result = img2.copy().astype(np.uint8)
    result[mask, 0] = 255  # Red channel
    result[mask, 1] = 0    # Green channel
    result[mask, 2] = 0    # Blue channel

    Image.fromarray(result).save(output_path)


def main():
    parser = argparse.ArgumentParser(
        description="Compare two images using SSIM for visual regression testing"
    )
    parser.add_argument("captured", help="Path to captured screenshot")
    parser.add_argument("reference", help="Path to reference screenshot")
    parser.add_argument(
        "--threshold", type=float, default=0.85,
        help="Minimum SSIM score to pass (0.0-1.0, default: 0.85)"
    )
    parser.add_argument(
        "--diff-output", default=None,
        help="Path to save diff image (optional)"
    )
    args = parser.parse_args()

    # Validate inputs
    if not Path(args.captured).exists():
        print(f"ERROR: Captured image not found: {args.captured}", file=sys.stderr)
        sys.exit(2)
    if not Path(args.reference).exists():
        print(f"ERROR: Reference image not found: {args.reference}", file=sys.stderr)
        sys.exit(2)

    # Load and compare
    img1, img2 = load_and_resize(args.captured, args.reference)
    ssim_score = compute_ssim(img1, img2)

    # Generate diff if requested
    if args.diff_output:
        generate_diff_image(img1, img2, args.diff_output)
        print(f"Diff image saved to: {args.diff_output}")

    # Report result
    passed = ssim_score >= args.threshold
    status = "PASS" if passed else "FAIL"
    print(f"SSIM: {ssim_score:.4f} (threshold: {args.threshold}) [{status}]")

    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
