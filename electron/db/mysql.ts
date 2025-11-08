import mysql from "mysql2/promise";

let connection: mysql.Connection | null = null;

export async function connect(config: any){
    connection = await mysql.createConnection({
        host: config.host,
        port: config.port,
        user: config.user,
        password: config.password,
        database: config.database
    });

    return { success : true };
}

export async function query(sql: string){
    if(!connection) throw new Error("MySQL not connected");

    const [rows, fields] = await connection.query(sql);
    return {
        rows,
        fields: fields.map((f: any) => f.name),
    };
}

export async function disconnect(){
    if (connection) {
        await connection.end();
        connection = null;
    }
}
