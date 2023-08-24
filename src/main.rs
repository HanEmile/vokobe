/*
pull the std into scope and inline it so that we get documentation for it,
even when running offline
*/
#[doc(inline)]
pub use std;

use std::path::{Path, PathBuf};
use std::io::{self, Read, Write, BufRead, BufReader};
use std::fs::{self, File};
use std::time;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "vokobe", about = "A static site generator")]
struct Opt {
    /// Input path 
    #[structopt(parse(from_os_str))]
    input_path: PathBuf,

    /// Output path
    #[structopt(parse(from_os_str))]
    output_path: PathBuf,

    /// Site name (e.g. emile.space)
    site_name: String,

    /// Activate sending analytics to stats.emile.space
    // -a and --analytics will be generated
    // analytics are sent to stats.emile.space
    #[structopt(short, long)]
    analytics: bool,
}

fn main() -> std::io::Result<()> {

    let opt = Opt::from_args();

    let in_path = opt.input_path;
    let output_path = opt.output_path;

    // read the style
    let style_path = Path::new(&in_path).join("style.css");
    let mut style_file = File::open(style_path)
        .expect("could not open style file");
    let mut style = String::new();
    style_file.read_to_string(&mut style)
        .expect("could not read style file to string");

    // read all dirs in the input path
    let pathes = recursive_read_dir(&in_path, false)?;

    println!("Got {} files", pathes.len());

    for path in pathes {
        let stripped_path = path.strip_prefix(&in_path)
            .expect(format!(
                "could not strip the in_path prefix: {:?}", in_path).as_str());

        // copy images and other files to the output folder
        if path.is_file() {

            // define the source and destination
            let src = Path::new(&in_path).join(stripped_path);
            let dst = Path::new(&output_path).join(stripped_path);

            // define the destination folder (the dst path without the file) and create it
            let mut dst_folder = dst.clone();
            dst_folder.pop(); // remove the file itself from the path
            fs::create_dir_all(dst_folder)?;

            // copy the file to the destination
            std::fs::copy(src, dst.as_path())?;
        }

        if stripped_path.ends_with("README.md") {

            // define the "raw" path (no infile prefix, no file)
            let mut ancestors = stripped_path.ancestors();
            ancestors.next();

            let raw_path = ancestors.next()
                .expect("could not extract next ancestor");

            // out + rawpath
            let index_path = output_path.join(raw_path);

            // (out + rawpath) + "index.html"
            let index_file = index_path.join("index.html");

            // - create the dir for the index.html as well as the index.html
            // itself
            fs::create_dir_all(index_path)?;
            let mut file = File::create(&index_file)?;

            // this is the main block calling all other smaller functions. The
            // whole output is compsed here
            write_header(&mut file, &opt.site_name, &style)?;
            write_body_start(&mut file, &opt.site_name)?;
            write_nav(&mut file, in_path.as_path(), raw_path, opt.analytics)?;
            write_same_level(&mut file, in_path.as_path(), raw_path)?;
            write_readme_content(&mut file, in_path.as_path(), raw_path)?;
            write_footer(&mut file)?;

            file.write_all("".as_bytes())?;
        }

    }

    Ok(())
}

/// Write the html header including the style file
/// TODO: Don't add the style file into each compiled html output, as the
/// style can be included allowing the user to cache the style file in their
/// browser.
fn write_header(file: &mut File, site_name: &String, style: &String) -> std::io::Result<()>{

    // write the header including the style file
    file.write_all(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>

  <style>
  {}
  </style>
</head>
    "#, site_name, style).as_bytes())?;

    Ok(())
}

/// write the start of the html body tag and the header linking back to the
/// site itself.
fn write_body_start(file: &mut File, site_name: &String) -> std::io::Result<()>{
    file.write_all(format!(r#"
<body>
  <header>
    <a href="/">{}</a>
  </header>"#, site_name).as_bytes())?;

    Ok(())
}

/// Write the navigation section to the given file
fn write_nav(file: &mut File, in_path: &Path, raw_path: &Path, analytics: bool)
    -> std::io::Result<()> {

    if analytics == true {
        /*
        file.write_all(format!(r#"
  <img src="https://stats.emile.space/count?p=/{}">
  <nav>
    <ul>"#, raw_path.to_str().unwrap()).as_bytes())?;
        */
        file.write_all(format!(r#"
  <nav>
    <ul>"#,).as_bytes())?;
    } else {
        file.write_all(format!(r#"
  <nav>
    <ul>"#).as_bytes())?;
    }

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
    <ul style="float: right">
        <li>{:?}</li>
        <li>
            <a href="README.md">.md</a>
        </li>
    </ul>
  </nav>"#, in_path.metadata()?.modified()?.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()).as_bytes())?;
    ////////////////////////////////////////////////////////////////////////////

    Ok(())
}


fn write_same_level(file: &mut File, in_path: &Path, raw_path: &Path)
    -> std::io::Result<()> {

    let search_path = Path::new(in_path).join(raw_path);

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    let mut vertical: bool = false;
    let mut show_files: bool = false;

    for entry in fs::read_dir(search_path)? {
        let path = &entry?.path();

        if path.is_dir() {
            dirs.push(path.to_path_buf());
        }
        if path.is_file() {
            files.push(path.to_path_buf());
            if path.file_name().unwrap() == "vertical" {
                vertical = true;
            }
            if path.file_name().unwrap() == "show_files" {
                show_files = true;
            }
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

    // counting the occurrence of `---`
    let mut hrule_count = 0;
    let mut in_yaml_metadata_block= false;

    let mut level_1_heading_num = 0;
    let mut level_2_heading_num = 0;
    let mut level_3_heading_num = 0;
    let mut level_4_heading_num = 0;
    let mut level_5_heading_num = 0;

    // cheap markdown 2 html converter
    for line in readme.split('\n') {

        // 1 == 2, as I'm not sure how to comment out the file write 5 lines or so below
        if in_yaml_metadata_block && 1 == 2 {
            // if we find the end of the yaml metadata block, break this
            if line.starts_with("---") {
                in_yaml_metadata_block = false;
                continue
            } else {
                file.write_all(format!(r##"yaml_line: {}
"##, line).as_bytes())?;
                continue
            }
        }

        // if we've got a horizontal rule, it can be two things: the start and
        // end of a yaml-metadata block or an actual horizontal rule.
        //
        // If it's yaml metadata, read it all, but don't print it, store it
        // for later
        // If it's a horizontal rule, print the horizontal rule
        if line.starts_with("---") {

            // store the yaml metadata
            if hrule_count == 0 {
                in_yaml_metadata_block = true;
                continue
            }                 
            hrule_count += 1;

            // print the horizontal rule
            file.write_all(format!(r##"
            <hr>"##).as_bytes())?;

        } else if line.starts_with("#####") {
            let heading = line.get(6..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            level_5_heading_num += 1;

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h5><a href="#{a}">{h1}.{h2}.{h3}.{h4}.{h5}. {b}</a></h3>
            <pre>"##,
                a = heading_sanitized,
                b = heading,
                h1 = level_1_heading_num,
                h2 = level_2_heading_num,
                h3 = level_3_heading_num,
                h4 = level_4_heading_num,
                h5 = level_5_heading_num,
            ).as_bytes())?;

        } else if line.starts_with("####") {
            let heading = line.get(5..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            level_4_heading_num += 1;
            level_5_heading_num = 0;

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h4><a href="#{a}">{h1}.{h2}.{h3}.{h4}. {b}</a></h3>
            <pre>"##,
                a = heading_sanitized,
                b = heading,
                h1 = level_1_heading_num,
                h2 = level_2_heading_num,
                h3 = level_3_heading_num,
                h4 = level_4_heading_num,
            ).as_bytes())?;

        } else if line.starts_with("###") {
            let heading = line.get(4..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            level_3_heading_num += 1;
            level_4_heading_num = 0;
            level_5_heading_num = 0;

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h3><a href="#{a}">{h1}.{h2}.{h3}. {b}</a></h3>
            <pre>"##,
                a = heading_sanitized,
                b = heading,
                h1 = level_1_heading_num,
                h2 = level_2_heading_num,
                h3 = level_3_heading_num,
            ).as_bytes())?;

        } else if line.starts_with("##") {
            let heading = line.get(3..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            level_2_heading_num += 1;
            level_3_heading_num = 0;
            level_4_heading_num = 0;
            level_5_heading_num = 0;

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h2><a href="#{a}">{h1}.{h2}. {b}</a></h2>
            <pre>"##,
                a = heading_sanitized,
                b = heading,
                h1 = level_1_heading_num,
                h2 = level_2_heading_num,
            ).as_bytes())?;

        } else if line.starts_with("#") {
            let heading = line.get(2..).unwrap();
            let heading_sanitized = sanitize(heading.to_string());

            level_1_heading_num += 1;
            level_2_heading_num = 0;
            level_3_heading_num = 0;
            level_4_heading_num = 0;
            level_5_heading_num = 0;

            file.write_all(format!(r##"</pre>
            <span id="{a}"></span>
            <h1><a href="#{a}">{h1}. {b}</a></h1>
            <pre>"##,
                a = heading_sanitized,
                b = heading,
                h1 = level_1_heading_num
            ).as_bytes())?;

        } else if line.starts_with("> ") {
            let line = line.replace("<", "&lt");
            let line = line.get(2..).unwrap();
            file.write_all(format!("</pre><pre class=\"code\">{}</pre><pre>\n", line).as_bytes())?;

        } else if line.starts_with(":::tree") {

            // TODO: add some parameter controlling if the list is ascending or descending (reverse the list before writing)

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

                // convert the path to a string, check if it contains a hidden
                // path by checking if it contains a `/.`, if so, skip this one
                if String::from(path.to_str().unwrap()).contains("/.") {
                    continue
                }
                if String::from(path.to_str().unwrap()).starts_with(".") {
                    continue
                }
                println!("[i] {:?}", path);

                // write the link and the entry name to the file
                let link = Path::new(raw_path).join(path);
                let name = path.file_name().unwrap().to_str().unwrap();

                // count the amount of segments in the path and write spaces for
                // each
                let segments = path.iter().count();
                for _ in 0..(segments-1) {
                    file.write_all(r#"    "#.as_bytes())?;
                }

                file.write_all(
                    format!("<a href=\"/{}\">{}</a>\n",
                        link.display(), name, 
                        ).as_bytes()
                )?;
            }

        } else if line.starts_with(":::toc") {

            // TODO: depth parameter for controlling the depth of the table of contents

            let mut level_1_num = 0;
            let mut level_2_num = 0;
            let mut level_3_num = 0;
            let mut level_4_num = 0;
            let mut level_5_num = 0;

            for line in readme.split('\n') {
                if line.starts_with("#####") {
                    let line = line.get(6..).unwrap();
                    // trim the line to remove the trailing whitespace
                    let line = line.trim();
                    level_5_num += 1;
                    file.write_all(
                        format!(
                            r##"           <a href="#{}">{}.{}.{}.{}.{}. {}</a>
"##,
                            sanitize(line.to_string()),
                            level_1_num,
                            level_2_num,
                            level_3_num,
                            level_4_num,
                            level_5_num,
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("####") {
                    let line = line.get(5..).unwrap();
                    // trim the line to remove the trailing whitespace
                    let line = line.trim();
                    level_4_num += 1;
                    level_5_num = 0;
                    file.write_all(
                        format!(
                            r##"         <a href="#{}">{}.{}.{}.{}. {}</a>
"##,
                            sanitize(line.to_string()),
                            level_1_num,
                            level_2_num,
                            level_3_num,
                            level_4_num,
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("###") {
                    let line = line.get(4..).unwrap();
                    // trim the line to remove the trailing whitespace
                    let line = line.trim();
                    level_3_num += 1;
                    level_4_num = 0;
                    level_5_num = 0;
                    file.write_all(
                        format!(
                            r##"       <a href="#{}">{}.{}.{}. {}</a>
"##,
                            sanitize(line.to_string()),
                            level_1_num,
                            level_2_num,
                            level_3_num,
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("##") {
                    let line = line.get(3..).unwrap();
                    let line = line.trim();
                    level_2_num += 1;
                    level_3_num = 0;
                    level_4_num = 0;
                    level_5_num = 0;

                    file.write_all(
                        format!(
                            //r##"    <a href="#{}">{}.{}. {}</a>
                            r##"    <a href="#{}">{}.{}. {}</a>
"##,
                            sanitize(line.to_string()),
                            level_1_num,
                            level_2_num,
                            line
                        ).as_bytes()
                    )?;
                } else if line.starts_with("#") {
                    let line = line.get(2..).unwrap();
                    let line = line.trim();
                    level_1_num += 1;
                    level_2_num = 0;
                    level_3_num = 0;
                    level_4_num = 0;
                    level_5_num = 0;

                    file.write_all(
                        format!(
                            r##"<a href="#{}">{}. {}</a>
"##,
                            sanitize(line.to_string()),
                            level_1_num,
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
    </pre>
<a href="https://chaos.social/@hanemile.rss" target="_blank" rel="noopener" class="icon"><img class="webring" src="/rss.svg" alt="rss feed of @hanemile@chaos.social mastodon" height="32px"/></a>
<a href="https://lieu.cblgh.org/" target="_blank" rel="noopener" class="icon"><img class="webring" src="/lieu.svg" alt="lieu webring search engine" height="32px"/></a>
<a href="https://webring.xxiivv.com/#emile" target="_blank" rel="noopener" class="icon"><img class="webring" src="/webring.svg" alt="XXIIVV webring" height="32px"/></a>
<a rel="me" href="https://chaos.social/@hanemile" target="_blank" class="icon"><img class="webring" src="/mastodon.svg" alt="mastodon" height="32px"/></a>
    <pre>emile - {:?} - generated using <a href="https://github.com/hanemile/vokobe">vokobe {:?}</a><pre>
</body>
</html>
"#,
    time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).unwrap(),
    env!("CARGO_PKG_VERSION")
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

    if dir.starts_with(".") {
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

        // skip hidden folders
        if path.starts_with(".") {
            //continue 'outer;
            break 'outer;
        }
        if dir.starts_with(".") {
            //continue 'outer;
            break 'outer;
        }

        // check if the current entry is part of the gitignore, if so, skip it
        for gitignore_entry in &gitignore_entries {
            if gitignore_entry.to_str() == Some("") {
                continue;
            }
            if path.ends_with(gitignore_entry) {
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
