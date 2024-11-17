from collections import OrderedDict
import base64
from io import BytesIO
from pathlib import Path
from typing import NamedTuple

from flask import Flask, request
from flask import render_template

from PIL import Image

from avulto import DME, DMI, Dir

app = Flask(__name__)

Entry = NamedTuple(
    "Entry", [("path", str), ("display_name", str), ("description", str)]
)
ImageFile = NamedTuple(
    "ImageFile", [("path", str), ("dmi", DMI), ("image", Image.Image)]
)

ROOT_PATH = Path("~/ExternalRepos/third_party/github/Paradise/").expanduser()
ENV_FILE = DME.from_file(ROOT_PATH / "paradise.dme")
PATHS = set(ENV_FILE.typesof("/obj"))

DICTIONARY = dict()
for path in PATHS:
    if path == "/obj":
        continue
    typedecl = ENV_FILE.type_decl(path)
    DICTIONARY[path] = Entry(
        path=path,
        display_name=typedecl.value("name"),
        description=typedecl.value("desc"),
    )

IMAGE_CACHE = dict()


def get_image_file(path) -> ImageFile:
    if path not in IMAGE_CACHE:
        print(f"get_image_file(path={path})")
        dmi = DMI.from_file(ROOT_PATH / path)
        image = Image.open(ROOT_PATH / path)
        IMAGE_CACHE[path] = ImageFile(path, dmi, image)

    return IMAGE_CACHE[path]


@app.route("/")
def root():
    return render_template("root.html")


@app.route("/search", methods=["POST"])
def search():
    s = request.form["search"]
    rows = []
    for k, v in DICTIONARY.items():
        if len(rows) < 10:
            if (
                s.lower() in str(k).lower()
                or (v.display_name and s.lower() in v.display_name.lower())
                or (v.description and s.lower() in v.description.lower())
            ):
                rows.append(
                    f"<tr><td><a href='/lookup{k}'>{v.display_name}</a></td><td>{v.description}</td><td>{k}</td></tr>"
                )
        else:
            break

    return "\n".join(rows)


@app.route("/lookup/<path:obj>")
def lookup(obj):
    obj = "/" + obj
    typedecl = ENV_FILE.type_decl(obj)
    name = typedecl.value("name")
    if not name:
        name = obj

    sections = OrderedDict()

    if typedecl.value("icon") and typedecl.value("icon_state"):
        image_file = get_image_file(typedecl.value("icon"))
        icon_state = image_file.dmi.state(typedecl.value("icon_state"))
        rect = icon_state.rect(Dir.SOUTH, 0)
        image_data = image_file.dmi.data_rgba8(rect)
        image = Image.frombytes("RGBA", size=(rect.width, rect.height), data=image_data)
        buffered = BytesIO()
        image.save(buffered, format="PNG")
        img_str = base64.b64encode(buffered.getvalue())
        image_string = "data:image/png;base64," + img_str.decode("utf-8")
        sections["Appearance"] = f"<img class='zoom_icon' src='{image_string}' />"

    if typedecl.value("desc"):
        sections["Description"] = typedecl.value("desc")

    origin_tech = typedecl.value("origin_tech")
    if origin_tech:
        techs = dict([x.split("=") for x in origin_tech.split(";")])
        sections["Research Breakdown"] = "<br />".join(
            [f"{k}: {techs[k]}" for k in sorted(techs)]
        )

    tech_data = f"Type: <tt>{obj}</tt>"
    tech_data += f"<br />Icon file: <tt>{typedecl.value('icon')}</tt>"
    tech_data += f"<br />Icon state: <tt>{typedecl.value('icon_state')}</tt>"
    sections["Technical Data"] = tech_data

    return render_template("entry.html", obj=obj, title=name, sections=sections)
