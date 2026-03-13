
import asyncio
import os
import asyncpg

async def check_warehouses():
    conn = await asyncpg.connect(os.environ.get("DATABASE_URL"))
    rows = await conn.fetch("SELECT id, code, name, is_active FROM warehouses")
    print(f"Total warehouses: {len(rows)}")
    for row in rows:
        print(f"ID: {row['id']}, Code: {row['code']}, Name: {row['name']}, Active: {row['is_active']}")
    
    rows_sl = await conn.fetch("SELECT id, warehouse_id, code, name FROM storage_locations")
    print(f"\nTotal storage locations: {len(rows_sl)}")
    for row in rows_sl:
        print(f"ID: {row['id']}, WH_ID: {row['warehouse_id']}, Code: {row['code']}, Name: {row['name']}")
    
    await conn.close()

if __name__ == "__main__":
    asyncio.run(check_warehouses())
