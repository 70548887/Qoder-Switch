use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

fn main() {
    let db_path = PathBuf::from(std::env::var("APPDATA").unwrap())
        .join("Qoder")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    
    println!("数据库路径: {:?}", db_path);
    println!("文件存在: {}", db_path.exists());
    
    if !db_path.exists() {
        println!("数据库文件不存在！");
        return;
    }
    
    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).expect("打开数据库失败");
    
    // 查询所有聊天历史 key
    let mut stmt = conn.prepare(
        "SELECT key, length(value) as val_len FROM ItemTable WHERE key LIKE 'lingma.chat.localHistory.%'"
    ).expect("准备语句失败");
    
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    }).expect("查询失败");
    
    println!("\n=== 聊天历史记录 ===");
    let mut count = 0;
    for row in rows {
        if let Ok((key, len)) = row {
            println!("Key: {} | 数据长度: {} 字节", key, len);
            count += 1;
        }
    }
    println!("\n共找到 {} 个工作区有聊天记录", count);
    
    // 对每个有记录的 key，解析并打印对话数量和标题
    let mut stmt2 = conn.prepare(
        "SELECT key, value FROM ItemTable WHERE key LIKE 'lingma.chat.localHistory.%'"
    ).expect("准备语句失败");
    
    let rows2 = stmt2.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).expect("查询失败");
    
    println!("\n=== 详细信息 ===");
    for row in rows2 {
        if let Ok((key, value)) = row {
            let workspace_id = key.strip_prefix("lingma.chat.localHistory.").unwrap_or(&key);
            if let Ok(chats) = serde_json::from_str::<Vec<serde_json::Value>>(&value) {
                println!("\n工作区 ID: {}", workspace_id);
                println!("  对话数量: {}", chats.len());
                for (i, chat) in chats.iter().take(5).enumerate() {
                    let title = chat.get("title").and_then(|t| t.as_str()).unwrap_or("(无标题)");
                    let ts = chat.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);
                    println!("  {}. {} (时间戳: {})", i+1, title, ts);
                }
                if chats.len() > 5 {
                    println!("  ... 还有 {} 条", chats.len() - 5);
                }
            }
        }
    }
}
