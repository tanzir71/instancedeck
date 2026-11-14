"""One-time format conversion of the exact installed, OFL-licensed design fonts."""
from pathlib import Path
from shutil import copyfile
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

root = Path(__file__).resolve().parents[1]
destination = root / "apps/site/assets/og-fonts"
destination.mkdir(parents=True, exist_ok=True)
for package, source, target, weight in [
    ("source-serif-4", "source-serif-4-latin-standard-normal.woff2", "serif.ttf", 400),
    ("ibm-plex-sans", "ibm-plex-sans-latin-wght-normal.woff2", "sans.ttf", 500),
]:
    package_dir = root / "packages/ui/node_modules/@fontsource-variable" / package
    font = TTFont(package_dir / "files" / source)
    font = instantiateVariableFont(font, {"wght": weight}, inplace=True)
    font.flavor = None
    font.save(destination / target)
    copyfile(package_dir / "LICENSE", destination / f"{package}-LICENSE.txt")
