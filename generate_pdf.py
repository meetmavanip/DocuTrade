from fpdf import FPDF

class PDF(FPDF):
    def header(self):
        self.set_font('Arial', 'B', 15)
        self.cell(0, 10, 'DocuTrade - Startup Instructions', 0, 1, 'C')
        self.ln(10)

    def chapter_title(self, num, label):
        self.set_font('Arial', 'B', 12)
        self.set_fill_color(200, 220, 255)
        self.cell(0, 6, f'Step {num} : {label}', 0, 1, 'L', 1)
        self.ln(4)

    def chapter_body(self, body):
        self.set_font('Arial', '', 12)
        self.multi_cell(0, 8, body)
        self.ln()

pdf = PDF()
pdf.add_page()

pdf.chapter_title(1, 'Start the Database (PostgreSQL)')
pdf.chapter_body(
    "Before anything else, your database must be running.\n"
    "- Open pgAdmin 4 or your Postgres service and ensure it is connected.\n"
    "- Ensure that the 'docutrade' database exists."
)

pdf.chapter_title(2, 'Start the Backend API (Rust)')
pdf.chapter_body(
    "The backend must be running for the frontend to save or fetch data.\n"
    "- Open a new terminal (Command Prompt or PowerShell).\n"
    "- Navigate to your backend folder: cd OneDrive\\Desktop\\DocuTrade\\backend\n"
    "- Run the server: cargo run\n"
    "(You should see a message saying 'Listening on 0.0.0.0:3000')"
)

pdf.chapter_title(3, 'Start the Frontend Website')
pdf.chapter_body(
    "Finally, you need to serve the HTML/JS/CSS files so you can view the website in your browser.\n"
    "- Open a SECOND (separate) terminal.\n"
    "- Navigate to your frontend folder: cd OneDrive\\Desktop\\DocuTrade\\frontend\n"
    "- Start a simple web server (using Python): python -m http.server 8080\n"
    "- Open your browser and go to: http://localhost:8080/login.html\n\n"
    "Once all three of these are running, the entire project is online! "
    "You can log in, create shipments, and view your profile successfully."
)

pdf.output('DocuTrade_Startup_Instructions.pdf', 'F')
print("PDF created successfully.")
