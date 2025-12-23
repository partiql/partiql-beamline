// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting-started/what-is-beamline.html"><strong aria-hidden="true">1.</strong> What is Beamline?</a></li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">2.</strong> Installation and Setup</a></li><li class="chapter-item expanded "><a href="getting-started/first-generation.html"><strong aria-hidden="true">3.</strong> Your First Data Generation</a></li><li class="chapter-item expanded affix "><li class="part-title">Understanding the Basics</li><li class="chapter-item expanded "><a href="basics/core-concepts.html"><strong aria-hidden="true">4.</strong> Core Concepts</a></li><li class="chapter-item expanded "><a href="basics/reproducible-generation.html"><strong aria-hidden="true">5.</strong> Reproducible Generation</a></li><li class="chapter-item expanded "><a href="basics/scripts-and-processes.html"><strong aria-hidden="true">6.</strong> Scripts and Processes</a></li><li class="chapter-item expanded affix "><li class="part-title">Data Generation</li><li class="chapter-item expanded "><a href="data-generation/overview.html"><strong aria-hidden="true">7.</strong> Overview</a></li><li class="chapter-item expanded "><a href="data-generation/generator-types.html"><strong aria-hidden="true">8.</strong> Generator Types</a></li><li class="chapter-item expanded "><a href="data-generation/datasets.html"><strong aria-hidden="true">9.</strong> Datasets</a></li><li class="chapter-item expanded "><a href="data-generation/scripts.html"><strong aria-hidden="true">10.</strong> Scripts</a></li><li class="chapter-item expanded "><a href="data-generation/static-data.html"><strong aria-hidden="true">11.</strong> Static Data</a></li><li class="chapter-item expanded "><a href="data-generation/output-formats.html"><strong aria-hidden="true">12.</strong> Output Formats</a></li><li class="chapter-item expanded "><a href="data-generation/nullability.html"><strong aria-hidden="true">13.</strong> Nullability</a></li><li class="chapter-item expanded affix "><li class="part-title">Query Generation</li><li class="chapter-item expanded "><a href="query-generation/overview.html"><strong aria-hidden="true">14.</strong> Overview</a></li><li class="chapter-item expanded "><a href="query-generation/basic-queries.html"><strong aria-hidden="true">15.</strong> Basic Queries</a></li><li class="chapter-item expanded "><a href="query-generation/advanced-patterns.html"><strong aria-hidden="true">16.</strong> Advanced Patterns</a></li><li class="chapter-item expanded "><a href="query-generation/parameterization.html"><strong aria-hidden="true">17.</strong> Parameterization</a></li><li class="chapter-item expanded affix "><li class="part-title">Schema</li><li class="chapter-item expanded "><a href="schema/understanding-shapes.html"><strong aria-hidden="true">18.</strong> Understanding Shapes</a></li><li class="chapter-item expanded "><a href="schema/shape-inference.html"><strong aria-hidden="true">19.</strong> Shape Inference</a></li><li class="chapter-item expanded "><a href="schema/output-formats.html"><strong aria-hidden="true">20.</strong> Output Formats</a></li><li class="chapter-item expanded affix "><li class="part-title">CLI</li><li class="chapter-item expanded "><a href="cli/overview.html"><strong aria-hidden="true">21.</strong> Overview</a></li><li class="chapter-item expanded "><a href="cli/data-commands.html"><strong aria-hidden="true">22.</strong> Data Commands</a></li><li class="chapter-item expanded "><a href="cli/query-commands.html"><strong aria-hidden="true">23.</strong> Query Commands</a></li><li class="chapter-item expanded "><a href="cli/shape-commands.html"><strong aria-hidden="true">24.</strong> Shape Commands</a></li><li class="chapter-item expanded "><a href="cli/database-commands.html"><strong aria-hidden="true">25.</strong> Database Commands</a></li><li class="chapter-item expanded affix "><li class="part-title">Database</li><li class="chapter-item expanded "><a href="database/overview.html"><strong aria-hidden="true">26.</strong> Overview</a></li><li class="chapter-item expanded "><a href="database/catalogs.html"><strong aria-hidden="true">27.</strong> Catalogs</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples and Tutorials</li><li class="chapter-item expanded "><a href="examples/sensors.html"><strong aria-hidden="true">28.</strong> Sensor Data Tutorial</a></li><li class="chapter-item expanded "><a href="examples/ecommerce.html"><strong aria-hidden="true">29.</strong> E-commerce</a></li><li class="chapter-item expanded "><a href="examples/financial.html"><strong aria-hidden="true">30.</strong> Financial</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
