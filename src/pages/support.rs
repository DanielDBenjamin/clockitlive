use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Support() -> impl IntoView {
    view! {
        <section class="support-page">
            <header class="page-header">
                <div style="display:flex;align-items:center;gap:8px;">
                    <A href="/student/profile" attr:class="link">"←"</A>
                    <h1 class="page-title">"Student Support"</h1>
                </div>
                <p class="page-subtitle">"Need help? We're here to assist you with ClockIt!"</p>
            </header>

            <section class="support-section">
                <div class="support-section-header" style="display: flex; align-items: center; gap: 8px;">
                    <span class="support-icon">"📚"</span>
                    <h2 class="support-section-title">"Quick Help"</h2>
                </div>

                <div class="support-grid">
                    <div class="support-card">
                        <div class="support-card-icon">"📅"</div>
                        <h3 class="support-card-title">"Viewing Your Timetable"</h3>
                        <p class="support-card-text">
                            "Your personalized timetable displays all your lectures, tutorials, and lab sessions. Use the week/day view toggle to switch between different formats."
                        </p>
                    </div>

                    <div class="support-card">
                        <div class="support-card-icon">"🔔"</div>
                        <h3 class="support-card-title">"Notifications"</h3>
                        <p class="support-card-text">
                            "Enable notifications to receive reminders before your classes start. Go to Settings to customize your notification preferences."
                        </p>
                    </div>

                    <div class="support-card">
                        <div class="support-card-icon">"🔍"</div>
                        <h3 class="support-card-title">"Finding a Venue"</h3>
                        <p class="support-card-text">
                            "Not sure where your class is? Click on any lecture card to see the full venue details and location information."
                        </p>
                    </div>

                    <div class="support-card">
                        <div class="support-card-icon">"✏️"</div>
                        <h3 class="support-card-title">"Managing Your Schedule"</h3>
                        <ul class="support-card-list">
                            <li><strong>"Add Classes:"</strong>" Use the search function to find and add courses"</li>
                            <li><strong>"Remove Classes:"</strong>" Click the menu icon on any lecture card"</li>
                            <li><strong>"Update Profile:"</strong>" Keep your information current in Profile settings"</li>
                        </ul>
                    </div>
                </div>
            </section>

            <section class="support-section">
                <div class="support-section-header" style="display: flex; align-items: center; gap: 8px;">
                    <span class="support-icon">"❓"</span>
                    <h2 class="support-section-title">"Common Issues"</h2>
                </div>

                <div class="support-faq">
                    <div class="faq-item">
                        <h3 class="faq-question">"Can't see my timetable?"</h3>
                        <p class="faq-answer">
                            "Make sure you've enrolled in courses and refresh the page. If issues persist, contact your department."
                        </p>
                    </div>

                    <div class="faq-item">
                        <h3 class="faq-question">"Notifications not working?"</h3>
                        <p class="faq-answer">
                            "Check that notifications are enabled in your browser settings and within ClockIt."
                        </p>
                    </div>

                    <div class="faq-item">
                        <h3 class="faq-question">"Venue information missing?"</h3>
                        <p class="faq-answer">
                            "Some venues may not be updated yet. Check with your lecturer or the department office."
                        </p>
                    </div>

                    <div class="faq-item">
                        <h3 class="faq-question">"Schedule conflicts?"</h3>
                        <p class="faq-answer">
                            "Contact your academic advisor to resolve overlapping classes."
                        </p>
                    </div>
                </div>
            </section>

            <section class="support-section">
                <div class="support-section-header" style="display: flex; align-items: center; gap: 8px;">
                    <span class="support-icon">"📧"</span>
                    <h2 class="support-section-title">"Contact Support"</h2>
                </div>

                <div class="contact-grid">
                    <div class="contact-card">
                        <h3 class="contact-title">"Email Support"</h3>
                        <a href="mailto:support@clockit.com" class="contact-link">"support@clockit.com"</a>
                        <p class="contact-info">"Response Time: Within 24 hours"</p>
                    </div>

                    <div class="contact-card">
                        <h3 class="contact-title">"Student Services Office"</h3>
                        <p class="contact-info">"Room 101, Admin Building"</p>
                        <p class="contact-info">"Monday - Friday: 8:00 AM - 4:00 PM"</p>
                    </div>
                </div>
            </section>

            <section class="support-section">
                <div class="support-section-header" style="display: flex; align-items: center; gap: 8px;">
                    <span class="support-icon">"💡"</span>
                    <h2 class="support-section-title">"Feedback"</h2>
                </div>

                <div class="feedback-box">
                    <p class="feedback-text">
                        "Have suggestions to improve ClockIt? We'd love to hear from you! Send feedback to "
                        <a href="mailto:feedback@clockit.com" class="contact-link">"feedback@clockit.com"</a>
                        " or use the feedback form in Settings."
                    </p>
                </div>
            </section>

            <div class="pro-tip">
                <span class="pro-tip-icon">"💡"</span>
                <strong>"Pro Tip:"</strong>
                " Bookmark ClockIt on your phone for quick access to your schedule on the go!"
            </div>
        </section>
    }
}