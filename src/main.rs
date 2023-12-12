use tera::Tera;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    error::Error,
    fs::{self, create_dir_all},
    path::PathBuf,
    process::Command,
};
use reqwest::blocking::get;
use serde::Deserialize;
use rayon::prelude::*;

const TEMPLATE_DIR: &str = "templates/";
const DRAWING_SVG: &str = "drawing.svg";
const OUTPUT_DIR: &str = "output/";

#[derive(Debug, Deserialize, Clone)]
struct ProductData {
    product_id: i32,
    product_code: String,
    manufacturer_code: String,
    manufacturer_name: String,
    name: String,
    name_ryaku: String,
    barcode: String,
    return_period: String,
    volume: String,
    bulk_size: i32,
    sales_start_date: String,
    sales_end_date: String,
}

fn fetch_product_data() -> Result<Vec<ProductData>, Box<dyn Error>> {
    let url = "http://localhost:3000/products";
    let response = get(url)?;

    let products: Vec<ProductData> = response.json()?;

    if !products.is_empty() {
        Ok(products)
    } else {
        Err("No products found".into())
    }
}

fn render_svg(product_data: &ProductData) -> Result<(), Box<dyn Error>> {
    let mut tera = Tera::new("templates/**/*")?;
    tera.add_template_file(&format!("{}{}", TEMPLATE_DIR, DRAWING_SVG), Some(TEMPLATE_DIR))?;
    
    let mut context = tera::Context::new();
    context.insert("name", &product_data.name);
    context.insert("name_ryaku", &product_data.name_ryaku);
    context.insert("product_id", &product_data.product_id);
    context.insert("barcode", &product_data.barcode);
    context.insert("return_period", &product_data.return_period);
    context.insert("volume", &product_data.volume);
    context.insert("bulk_size", &product_data.bulk_size);
    context.insert("manufacturer_name", &product_data.manufacturer_name);
    context.insert("manufacturer_code", &product_data.manufacturer_code);
    context.insert("created_at", &product_data.sales_start_date);
    context.insert("updated_at", &product_data.sales_end_date);
    context.insert("code", &product_data.product_code);

    let svg = tera.render(DRAWING_SVG, &context)?;

    let rng = thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let output_path = PathBuf::from(OUTPUT_DIR);
    let output_svg = output_path.join(format!("{}.svg", random_string));
    let output_pdf = output_path.join(format!("{}.pdf", random_string));

    fs::write(&output_svg, svg)?;

    Command::new("Inkscape/bin/inkscape.exe")
        .arg(format!("--export-filename={}", output_pdf.to_string_lossy()))
        .arg(&output_svg)
        .output()?;

    Ok(())
}

fn main() {
    let output_path = PathBuf::from(OUTPUT_DIR);
    if let Err(err) = create_dir_all(&output_path) {
        eprintln!("Error creating directory: {}", err);
        return; // or handle this error according to your application's logic
    }

    match fetch_product_data() {
        Ok(products) => {
            products.par_iter().for_each(|product_data| {
                if let Err(err) = render_svg(product_data) {
                    eprintln!("Error rendering SVG for product: {}", err);
                }
            });
        }
        Err(err) => {
            eprintln!("Failed to fetch product data: {}", err);
        }
    }
}
