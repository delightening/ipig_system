use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::{handlers, AppState};

/// ERP 路由：倉庫、儲位、產品、SKU、交易夥伴、單據、庫存
pub fn routes() -> Router<AppState> {
    Router::new()
        // Warehouses
        .route(
            "/warehouses",
            get(handlers::list_warehouses).post(handlers::create_warehouse),
        )
        .route(
            "/warehouses/with-shelves",
            get(handlers::list_warehouses_with_shelves),
        )
        .route("/warehouses/import", post(handlers::import_warehouses))
        .route(
            "/warehouses/import/template",
            get(handlers::download_warehouse_import_template),
        )
        .route(
            "/warehouses/:id",
            get(handlers::get_warehouse)
                .put(handlers::update_warehouse)
                .delete(handlers::delete_warehouse),
        )
        .route("/warehouses/:id/delete", post(handlers::delete_warehouse))
        .route(
            "/warehouses/:id/layout",
            put(handlers::storage_location::update_warehouse_layout),
        )
        // Storage Locations
        .route(
            "/storage-locations",
            get(handlers::storage_location::list_storage_locations)
                .post(handlers::storage_location::create_storage_location),
        )
        .route(
            "/storage-locations/:id",
            get(handlers::storage_location::get_storage_location)
                .put(handlers::storage_location::update_storage_location)
                .delete(handlers::storage_location::delete_storage_location),
        )
        .route(
            "/storage-locations/:id/delete",
            post(handlers::storage_location::delete_storage_location),
        )
        .route(
            "/storage-locations/:id/inventory",
            get(handlers::storage_location::get_storage_location_inventory)
                .post(handlers::storage_location::create_storage_location_inventory_item),
        )
        .route(
            "/storage-locations/inventory/:item_id",
            put(handlers::storage_location::update_storage_location_inventory_item),
        )
        .route(
            "/storage-locations/inventory/:item_id/transfer",
            post(handlers::storage_location::transfer_storage_location_inventory),
        )
        .route(
            "/storage-locations/generate-code/:warehouse_id",
            get(handlers::storage_location::generate_storage_location_code),
        )
        // Products
        .route(
            "/products",
            get(handlers::list_products).post(handlers::create_product),
        )
        .route("/products/import", post(handlers::import_products))
        .route(
            "/products/import/check",
            post(handlers::check_product_import_duplicates),
        )
        .route(
            "/products/import/preview",
            post(handlers::preview_product_import),
        )
        .route(
            "/products/import/template",
            get(handlers::download_product_import_template),
        )
        .route(
            "/products/:id",
            get(handlers::get_product)
                .put(handlers::update_product)
                .delete(handlers::delete_product),
        )
        .route(
            "/products/:id/status",
            patch(handlers::update_product_status),
        )
        .route("/products/:id/delete", post(handlers::delete_product))
        .route(
            "/products/:id/hard-delete",
            post(handlers::hard_delete_product),
        )
        .route(
            "/categories",
            get(handlers::list_categories).post(handlers::create_category),
        )
        // SKU
        .route("/sku/categories", get(handlers::get_sku_categories))
        .route(
            "/sku/categories/tree",
            get(handlers::get_sku_categories_tree),
        )
        .route(
            "/sku/categories/:code",
            patch(handlers::update_sku_category).delete(handlers::delete_sku_category),
        )
        .route(
            "/sku/categories/:category_code/subcategories",
            get(handlers::get_sku_subcategories).post(handlers::create_sku_subcategory),
        )
        .route(
            "/sku/categories/:category_code/subcategories/:code",
            patch(handlers::update_sku_subcategory).delete(handlers::delete_sku_subcategory),
        )
        .route("/sku/generate", post(handlers::generate_sku))
        .route("/sku/validate", post(handlers::validate_sku))
        .route("/skus/preview", post(handlers::preview_sku))
        .route("/products/with-sku", post(handlers::create_product_with_sku))
        // Partners
        .route(
            "/partners",
            get(handlers::list_partners).post(handlers::create_partner),
        )
        .route("/partners/import", post(handlers::import_partners))
        .route(
            "/partners/import/template",
            get(handlers::download_partner_import_template),
        )
        .route(
            "/partners/generate-code",
            get(handlers::generate_partner_code),
        )
        .route(
            "/partners/:id",
            get(handlers::get_partner)
                .put(handlers::update_partner)
                .delete(handlers::delete_partner),
        )
        .route("/partners/:id/delete", post(handlers::delete_partner))
        // Documents
        .route(
            "/documents",
            get(handlers::list_documents).post(handlers::create_document),
        )
        .route(
            "/documents/:id",
            get(handlers::get_document)
                .put(handlers::update_document)
                .delete(handlers::delete_document),
        )
        .route("/documents/:id/delete", post(handlers::delete_document))
        .route("/documents/:id/submit", post(handlers::submit_document))
        .route("/documents/:id/approve", post(handlers::approve_document))
        .route("/documents/:id/cancel", post(handlers::cancel_document))
        .route(
            "/documents/:id/receipt-status",
            get(handlers::get_po_receipt_status),
        )
        // Inventory
        .route("/inventory/on-hand", get(handlers::get_inventory_on_hand))
        .route("/inventory/ledger", get(handlers::get_stock_ledger))
        .route("/inventory/low-stock", get(handlers::get_low_stock_alerts))
        .route(
            "/inventory/unassigned",
            get(handlers::get_unassigned_inventory),
        )
}
