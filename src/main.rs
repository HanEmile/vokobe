// pull the std into scope and inline it so that we get documentation for it,
// even when running offline
#[doc(inline)]
pub use std;

use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use std::fs::{self, File};
use std::time;

fn main() -> std::io::Result<()> {

    // the input and output pathes
    let in_path = Path::new("../emile.space/in").to_path_buf();
    let out_path = Path::new("../emile.space/out").to_path_buf();

    println!("inpath: {}", in_path.display());
    println!("outpath: {}", out_path.display());

    // read the style
    let mut style_file = File::open("./style.css")?;
    let mut style = String::new();
    style_file.read_to_string(&mut style)?;

    // read all dirs in the input path
    let pathes = recursive_read_dir(&in_path, false)?;

    println!("---");
    for path in pathes {
        println!("\n");
        println!("[i] {}", path.as_os_str().to_str().unwrap());

        let stripped_path = path.strip_prefix(&in_path)
            .expect(format!(
                "could not strip the in_path prefix: {:?}", in_path).as_str());

        // copy images and other files to the output folder
        if path.is_file() {

            // define the source and destination
            let src = Path::new(&in_path).join(stripped_path);
            let dst = Path::new(&out_path).join(stripped_path);

            // define the destination folder (the dst path without the file) and
            // create it
            let mut dst_folder = dst.clone();
            dst_folder.pop();
            fs::create_dir_all(dst_folder)?;

            // copy the file to the destination
            std::fs::copy(src, dst.as_path())?;
        }

        if stripped_path.ends_with("README.md") {
            println!("\tstripped_path: {:?}", stripped_path);

            // define the "raw" path (no infile prefix, no file)
            let mut ancestors = stripped_path.ancestors();
            ancestors.next();

            let raw_path = ancestors.next()
                .expect("could not extract next ancestor");
            println!("\traw_path: {:?}", raw_path);

            // out + rawpath
            let index_path = out_path.join(raw_path);
            println!("\tindex_path: {:?}", index_path);

            // (out + rawpath) + "index.html"
            let index_file = index_path.join("index.html");
            println!("\tindex_file: {:?}", index_file);

            // - create the dir for the index.html as well as the index.html itself
            fs::create_dir_all(index_path)?;
            let mut file = File::create(&index_file)?;

            write_header(&mut file, &style)?;
            write_body_start(&mut file)?;
            write_nav(&mut file, in_path.as_path(), raw_path)?;
            write_same_level(&mut file, in_path.as_path(), raw_path)?;
            write_readme_content(&mut file, in_path.as_path(), raw_path)?;
            write_footer(&mut file)?;

            file.write_all("".as_bytes())?;
        }

    }

    Ok(())
}

/// Write the html header including the style file
fn write_header(file: &mut File, style: &String) -> std::io::Result<()>{

    // write the header including the style file
    file.write_all(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>emile.space</title>

  <style>
  {}
  </style>
</head>
    "#, style).as_bytes())?;

    Ok(())
}

fn write_body_start(file: &mut File) -> std::io::Result<()>{
    file.write_all(format!(r#"
<body>
  <header>
    <a href="/">emile.space</a>
  </header>"#).as_bytes())?;

    Ok(())
}

/// Write the navigation section to the given file
fn write_nav(file: &mut File, in_path: &Path, raw_path: &Path)
    -> std::io::Result<()> {

    ////////////////////////////////////////////////////////////////////////////
    file.write_all(format!(r#"
  <nav>
    <ul>"#).as_bytes())?;
    ////////////////////////////////////////////////////////////////////////////

    // get the nav bar components
    let components = raw_path.components().collect::<Vec<_>>();
    
    // for each list of components (["a"], ["a", "b"], ["a", "b", "c"]), create
    // the path for the list, view all other dirs at that path and write the
    // result to the file
    let mut i = 0;
    let slice = components.as_slice();

    // for each navbar component
    for component in slice {

        // get the items belonging to that navbar item
        // (["a"], ["a", "b"], ["a", "b", "c"])
        let subpath_components = &slice[..i+1];
        i += 1;

        println!("\tsubpath_components:");
        subpath_components.iter().for_each(|c| {
            println!("\t\t{:?}", c);
        });

        let mut subpath_path = PathBuf::new();

        // push the inpath, so we've got a basis from where we can read the
        // subpath items
        // subpath_path = inpath + ???
        subpath_path.push(in_path);

        let mut nav_breadcrumb_link = PathBuf::new();

        // for each item in the subpath, push it into the subpath_path so that
        // in the end, we've got something like this:
        // "inpath" + "a" + "b" + "c"
        for subpath_component in subpath_components {
            subpath_path.push(subpath_component);
            nav_breadcrumb_link.push(subpath_component);
        }

        // make the nav_breadcrumb_link an absolute by prefixing it with a /
        // (this is in scope of the web-page, so this is find) and make it a
        // string
        let nav_breadcrumb_link_absolute 
            = Path::new("/")
                .join(nav_breadcrumb_link);

        let nav_breadcrumb_link
            = nav_breadcrumb_link_absolute.to_str().unwrap();

        // define the name of the breadcrumb
        let nav_breadcrumb_name = component.as_os_str().to_str().unwrap();

        ////////////////////////////////////////////////////////////////////////
        file.write_all(format!(r#"
        <li>
            <a href="{}">{}</a>
            <ul>"#, nav_breadcrumb_link, nav_breadcrumb_name).as_bytes())?;
        ////////////////////////////////////////////////////////////////////////

        // as we don't want to get the items for the individial entry, but on
        // the same level, we push a ".."
        // the subpath_path is now: inpath + subpath + ../
        subpath_path.push("..");

        println!("\t\tsubpath_path: {:?}", subpath_path);

        // read all dirs in the subpath_path, add them to the dirs vector, so
        // that we get a vector containing all the dirs we want
        let mut dirs = Vec::new();
        for entry in fs::read_dir(subpath_path)? {
            let path = &entry?.path();
            if path.is_dir() {
                dirs.push(path.to_path_buf());
            }
        }

        dirs.sort();

        // DROPDOWN
        // extract the link and name for each directory found
        for dir in dirs {
            let d = dir.canonicalize()?;
            let abs_inpath = in_path.canonicalize()?;

            let name = d.file_name().unwrap().to_str().unwrap();
            let rel_link 
                = d.strip_prefix(abs_inpath)
                    .expect(format!(
                        "could not strip the in_path prefix: {:?}",
                        d).as_str());

            let link = Path::new("/").join(rel_link);
            let link = link.as_path().to_str().unwrap();

            println!("\t\t\t{:?} {:?}", name, link);

            // don't add the current page to the dropdown, we're on it already!
            if name == nav_breadcrumb_name {
                continue
            }

            // don't add items starting with a dot to the dropdown, they're
            // hidden!
            if name.starts_with(".") {
                continue
            }

            ////////////////////////////////////////////////////////////////////
            file.write_all(format!(r#"
                <li><a href="{}">{}/</a></li>"#, link, name).as_bytes())?;
            ////////////////////////////////////////////////////////////////////
        }

        ////////////////////////////////////////////////////////////////////////
        file.write_all(r#"
            </ul>
        </li>"#.as_bytes())?;
        ////////////////////////////////////////////////////////////////////////
    }

    ////////////////////////////////////////////////////////////////////////////
    file.write_all(format!(r#"
    </ul>
  </nav>"#).as_bytes())?;
    ////////////////////////////////////////////////////////////////////////////

    Ok(())
}


fn write_same_level(file: &mut File, in_path: &Path, raw_path: &Path)
    -> std::io::Result<()> {

    let search_path = Path::new(in_path).join(raw_path);

    println!("\tsame_level:");
    println!("\t\t{:?}", search_path);

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    let mut vertical: bool = false;
    let mut show_files: bool = false;

    for entry in fs::read_dir(search_path)? {
        let path = &entry?.path();

        if path.is_dir() {
            dirs.push(path.to_path_buf());
            println!("\t\t\t{:?}", path);
        }
        if path.is_file() {
            files.push(path.to_path_buf());
            if path.file_name().unwrap() == "vertical" {
                vertical = true;
            }
            if path.file_name().unwrap() == "show_files" {
                show_files = true;
            }
            println!("\t\t\t{:?}", path);
        }
    }

    dirs.sort();
    files.sort();

    let in_path = in_path.canonicalize()?;

    if vertical == true {
        file.write_all(format!(r#"
  <ul class="vert">"#).as_bytes())?;
    } else {
        file.write_all(format!(r#"
  <ul>"#).as_bytes())?;
    }

    for dir in dirs {
        let dir = dir.canonicalize()?;
        let dir = dir.strip_prefix(&in_path)
            .expect("could not strip in_path prefix");

        let link = Path::new("/").join(dir);
        let link_str = link.as_path().to_str().unwrap();
        let name = link.file_name().unwrap().to_str().unwrap();

        if name.starts_with(".") {
            continue
        }

        file.write_all(format!(r#"
    <li><a href="{}">{}/</a></li>"#, link_str, name).as_bytes())?;
        println!("\t\t{} {}", link_str, name);
    }

    file.write_all(format!(r#"
  </ul>"#).as_bytes())?;

    if files.len() >= 1 && show_files == true {
        file.write_all(format!(r#"<br>
    <ul>"#).as_bytes())?;

        for f in files {
            let f = f.canonicalize()?;
            let f = f.strip_prefix(&in_path)
                .expect("could not strip in_path prefix");

            let link = Path::new("/").join(f);
            let link_str = link.as_path().to_str().unwrap();
            let name = link.file_name().unwrap().to_str().unwrap();

            if name == "README.md"
                || name == "show_files"
                || name.starts_with(".")
                {
                continue
            };

            file.write_all(format!(r#"
        <li><a href="{}">{}</a></li>"#, link_str, name).as_bytes())?;
            println!("\t\t{} {}", link_str, name);
        }

        file.write_all(format!(r#"
    </ul>"#).as_bytes())?;
    }


    Ok(())
}

fn write_readme_content(file: &mut File, in_path: &Path, raw_path: &Path) 
    -> std::io::Result<()> {

    // define the path of the README.md file
    let readme_file_path 
        = Path::new(in_path).join(raw_path).join("README.md");

    // open the file and read it as a string
    let mut readme_file = File::open(readme_file_path)?;
    let mut readme = String::new();
    readme_file.read_to_string(&mut readme)?;

    file.write_all(format!("<pre>").as_bytes())?;

    // cheap markdown 2 html converter
    for line in readme.split('\n') {

        if line.starts_with("###") {
            let heading = line.get(4..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h3><a href="#{a}">{b}</a></h3>
            <pre>"##, a = heading_sanitized, b = heading).as_bytes())?;

        } else if line.starts_with("##") {
            let heading = line.get(3..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h2><a href="#{a}">{b}</a></h2>
            <pre>"##, a = heading_sanitized, b = heading).as_bytes())?;

        } else if line.starts_with("#") {
            let heading = line.get(2..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h1><a href="#{a}">{b}</a></h1>
            <pre>"##, a = heading_sanitized, b = heading).as_bytes())?;

        } else if line.starts_with(">") {
            let line = line.get(1..).unwrap();
            file.write_all(format!("</pre><pre class=\"code\">{}</pre><pre>\n", line).as_bytes())?;

        } else if line.starts_with(":::tree") {

            // get all dirs in the current dir recursively
            let tree_files_path = Path::new(in_path).join(raw_path);
            let mut tree_files
                = recursive_read_dir(&tree_files_path, true)?;
            
            // sort them, otherwise we'll get complete chaos
            tree_files.sort();

            for path in tree_files {

                // strip the inpath prefix and raw_path prefix, as we don't need
                // them
                let path 
                    = path.strip_prefix(in_path)
                        .expect("could not strip in_file prefix")
                        .strip_prefix(raw_path)
                        .expect("could not strip raw_path prefix");

                // write the link and the entry name to the file
                let link = Path::new(raw_path).join(path);
                let name = path.file_name().unwrap().to_str().unwrap();

                if name.starts_with(".") {
                    continue
                }

                // count the amount of segments in the path and write spaces for
                // each
                let segments = path.iter().count();
                for _ in 0..(segments-1) {
                    file.write_all(r#"    "#.as_bytes())?;
                }

                // write the linke and the entry name to the file
                let link = Path::new(raw_path).join(path);
                file.write_all(
                    format!("<a href=\"/{}\">{}</a>\n",
                        link.display(), name, 
                        ).as_bytes()
                )?;
            }

        } else if line.starts_with(":::toc") {

            for line in readme.split('\n') {
                if line.starts_with("###") {
                    let line = line.get(4..).unwrap();
                    file.write_all(
                        format!(
                            r##"       <a href="#{}">{}</a>
"##,
                            sanitize(line.to_string()),
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("##") {
                    let line = line.get(3..).unwrap();
                    file.write_all(
                        format!(
                            r##"    <a href="#{}">{}</a>
"##,
                            sanitize(line.to_string()),
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("#") {
                    let line = line.get(2..).unwrap();
                    file.write_all(
                        format!(
                            r##"<a href="#{}">{}</a>
"##,
                            sanitize(line.to_string()),
                            line
                        ).as_bytes()
                    )?;
                }
            }

            
        } else {

            // for the case that nothing of the above matches, just write the
            // content into the html body as it is
            file.write_all(format!("{}\n", line).as_bytes())?;
        }
    }

    Ok(())
}

fn write_footer(file: &mut File) -> std::io::Result<()> {
    file.write_all(format!(r#"<br>
    <br>
    <br>
———
emile - {:?}
</body>
</html>
    <pre>"#,
    time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH).unwrap()
    ).as_bytes())?;

    Ok(())
}

/// sanitize the given string (to lower + space to hypen + keep only
/// [a-zA-Z0-9])
fn sanitize(input: String) -> String {
    let input = input.replace(" ", "-");

    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || c.eq(&'-'))
        .collect::<String>()
        .to_lowercase()
}

/// Return a list of all files in the directory, recursively.
fn recursive_read_dir(dir: &PathBuf, dir_only: bool) -> io::Result<Vec<PathBuf>> {
    // return an empty vec if the given path is not a directory
    if dir.is_dir() == false {
        return Ok(vec![]);
    }

    // get all entries in the gitignore file, if it exists
    let gitignore_entries: Vec<PathBuf> = gitignore_entries(&dir)?;

    // store the child pathes
    let mut entries: Vec<PathBuf> = Vec::new();
    
    // iterate over all items in the dir, pushing the dirs pathes to the dirs
    // vector for returning it
    'outer: for entry in fs::read_dir(dir)? {
        let dir_entry = &entry?;
        let path = dir_entry.path();

        // check if the current entry is part of the gitignore, if so, skip it
        for gitignore_entry in &gitignore_entries {
            if gitignore_entry.to_str() == Some("") {
                continue;
            }
            if path.ends_with(gitignore_entry) {
                println!("gitignore: gitignore_entry: {:?}", gitignore_entry);
                println!("gitignore: path: {:?}", path);
                continue 'outer;
            }
        }

        if dir_only == true {
            if path.is_dir() {
                entries.push(path.to_path_buf());
            }
        } else {
            entries.push(path.to_path_buf());
        }

        // recursively push all dirs from all children to the dirs vector
        let subdirs = recursive_read_dir(&path, dir_only)?;

        for subdir in subdirs {
            entries.push(subdir)
        }
    }

    // return the dirs, the ones from this folder and the ones from all child folders
    Ok(entries)
}

// try to open the gitignore file and read all entries from there.
fn gitignore_entries(dir: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let gitignore_path = Path::new(&dir)
        .join(Path::new(".gitignore"));

    let mut entries: Vec<PathBuf> = Vec::new();
    if let Ok(gitignore) = File::open(&gitignore_path) {
        let reader = BufReader::new(gitignore);

        for line in reader.lines() {
            entries.push(PathBuf::from(line?));
        }
    }

    Ok(entries)
}