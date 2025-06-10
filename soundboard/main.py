import flask
import os

# https://repl-pinger.quantumcodes.repl.co
# 30753

app = flask.Flask(
    "app",
    static_folder="memes",
    static_url_path="/memes/",
    template_folder="./templates",
)


def getmemes():
    return [
        x.replace(".png", "")
        for x in os.listdir("memes/img")
        if not x.endswith((".br", ".gz"))
    ]


@app.route("/")
def home():
    return flask.render_template("main.html", memes=getmemes())


def render_template_to_file(template_name, output_file, **context):
    """Renders a template and saves it to a file."""
    with app.app_context():  # Ensure an application context is available
        rendered = flask.render_template(template_name, **context)
        with open(output_file, "w") as file:
            file.write(rendered)
        print(f"Template saved to {output_file}")


# Save the rendered template to a file
if __name__ == "__main__":
    with app.app_context():  # Needed for loading templates outside routes
        # Example rendering with a template file
        render_template_to_file("main.html", "main-out.html", memes=getmemes())
    # check if the --no-server flag was passed
    if "--no-server" not in os.sys.argv:
        app.run("0.0.0.0")
