use crate::error::AppError;
use crate::models::invoice::InvoiceResponse;
use printpdf::*;
use std::io::BufWriter;

pub fn generate_pdf(invoice: &InvoiceResponse, user_name: &str) -> Result<Vec<u8>, AppError> {
    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Invoice {}", invoice.invoice_number),
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );

    let layer = doc.get_page(page1).get_layer(layer1);

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| AppError::PdfError(e.to_string()))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| AppError::PdfError(e.to_string()))?;

    // Title
    layer.use_text("INVOICE", 28.0, Mm(20.0), Mm(270.0), &font_bold);
    layer.use_text(
        &format!("#{}", invoice.invoice_number),
        14.0,
        Mm(20.0),
        Mm(258.0),
        &font,
    );

    // Status badge
    layer.use_text(
        &format!("Status: {}", invoice.status),
        10.0,
        Mm(150.0),
        Mm(270.0),
        &font,
    );

    // From section
    layer.use_text("FROM", 9.0, Mm(20.0), Mm(244.0), &font_bold);
    layer.use_text(user_name, 10.0, Mm(20.0), Mm(238.0), &font);

    // To section
    layer.use_text("TO", 9.0, Mm(110.0), Mm(244.0), &font_bold);
    layer.use_text(&invoice.client_name, 10.0, Mm(110.0), Mm(238.0), &font);
    layer.use_text(&invoice.client_email, 10.0, Mm(110.0), Mm(232.0), &font);
    if let Some(addr) = &invoice.client_address {
        layer.use_text(addr, 10.0, Mm(110.0), Mm(226.0), &font);
    }

    // Dates
    layer.use_text(
        &format!("Issue Date:  {}", invoice.issue_date),
        10.0,
        Mm(20.0),
        Mm(222.0),
        &font,
    );
    layer.use_text(
        &format!("Due Date:    {}", invoice.due_date),
        10.0,
        Mm(20.0),
        Mm(216.0),
        &font,
    );

    // Items table header
    let mut y: f32 = 204.0;
    layer.use_text("Description", 10.0, Mm(20.0), Mm(y), &font_bold);
    layer.use_text("Qty", 10.0, Mm(110.0), Mm(y), &font_bold);
    layer.use_text("Unit Price", 10.0, Mm(135.0), Mm(y), &font_bold);
    layer.use_text("Amount", 10.0, Mm(168.0), Mm(y), &font_bold);
    y -= 2.0;

    // Divider line
    let line_points = vec![
        (Point::new(Mm(20.0), Mm(y)), false),
        (Point::new(Mm(190.0), Mm(y)), false),
    ];
    let line = Line {
        points: line_points,
        is_closed: false,
    };
    layer.add_line(line);
    y -= 6.0;

    // Items rows
    for item in &invoice.items {
        layer.use_text(&item.description, 10.0, Mm(20.0), Mm(y), &font);
        layer.use_text(&format!("{:.2}", item.quantity), 10.0, Mm(110.0), Mm(y), &font);
        layer.use_text(
            &format!("${:.2}", item.unit_price),
            10.0,
            Mm(135.0),
            Mm(y),
            &font,
        );
        layer.use_text(
            &format!("${:.2}", item.amount),
            10.0,
            Mm(168.0),
            Mm(y),
            &font,
        );
        y -= 8.0;
    }

    // Totals section
    y -= 4.0;
    let line_points = vec![
        (Point::new(Mm(130.0), Mm(y)), false),
        (Point::new(Mm(190.0), Mm(y)), false),
    ];
    let line = Line {
        points: line_points,
        is_closed: false,
    };
    layer.add_line(line);
    y -= 6.0;

    layer.use_text("Subtotal:", 10.0, Mm(135.0), Mm(y), &font);
    layer.use_text(&format!("${:.2}", invoice.subtotal), 10.0, Mm(168.0), Mm(y), &font);
    y -= 8.0;

    if invoice.tax_rate > 0.0 {
        layer.use_text(
            &format!("Tax ({:.1}%):", invoice.tax_rate),
            10.0,
            Mm(135.0),
            Mm(y),
            &font,
        );
        layer.use_text(
            &format!("${:.2}", invoice.tax_amount),
            10.0,
            Mm(168.0),
            Mm(y),
            &font,
        );
        y -= 8.0;
    }

    layer.use_text("Total:", 12.0, Mm(135.0), Mm(y), &font_bold);
    layer.use_text(&format!("${:.2}", invoice.total), 12.0, Mm(168.0), Mm(y), &font_bold);

    // Notes
    if let Some(notes) = &invoice.notes {
        y -= 20.0;
        layer.use_text("Notes:", 10.0, Mm(20.0), Mm(y), &font_bold);
        y -= 8.0;
        layer.use_text(notes, 10.0, Mm(20.0), Mm(y), &font);
    }

    let mut bytes = Vec::new();
    doc.save(&mut BufWriter::new(&mut bytes))
        .map_err(|e| AppError::PdfError(e.to_string()))?;

    Ok(bytes)
}
