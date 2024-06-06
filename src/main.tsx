import React from 'react'
import {BrowserRouter, Route, Routes, useNavigate} from 'react-router-dom';
import ReactDOM from 'react-dom/client'
import $ from 'jquery'
import {Link, NavbarMenu, NavbarMenuItem, NavbarMenuToggle, NextUIProvider} from "@nextui-org/react";

import {Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@nextui-org/navbar";

import "./assets/scss/index.scss"
import Home from "./assets/pages/Home.tsx";
import About from "./assets/pages/About.tsx";
import ThemeSwitcher, {applyTheme} from "./assets/components/ThemeSwitcher.tsx";

applyTheme()

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <PageContent/>
        </BrowserRouter>
    </React.StrictMode>,
)


function PageContent() {
    const navigate = useNavigate();
    return (
        <NextUIProvider navigate={navigate}>
            <Nav/>
            <Routes>
                <Route>
                    <Route path="/" element={<Home/>}/>
                    <Route path="/about" element={<About/>}/>
                </Route>
            </Routes>
        </NextUIProvider>
    );
}


function Nav() {
    const [isMenuOpen, setIsMenuOpen] = React.useState(false);
    const pages = {
        "Home": "/",
        "About": "/about",
    };
    const menuItems = Object.keys(pages).map((item, index) => {
        const url = Object.values(pages)[index];
        const isCurrentPage = window.location.pathname === url;
        return (
            <NavbarMenuItem>
                <Link href={url} color={isCurrentPage ? "primary" : "foreground"} aria-current="page" size="lg" className="w-full">
                    {item}
                </Link>
            </NavbarMenuItem>
        );
    });


    return (
        <Navbar onMenuOpenChange={setIsMenuOpen}>
            <NavbarContent>
                <NavbarMenuToggle aria-label={isMenuOpen ? "Close menu" : "Open menu"} className="sm:hidden"/>
                <NavbarBrand>
                    <p className="font-bold text-inherit">Example</p>
                </NavbarBrand>
            </NavbarContent>

            <NavbarContent className="hidden sm:flex gap-4" justify="center">
                {menuItems}
            </NavbarContent>
            <NavbarContent justify="end">
                <NavbarItem>
                    <ThemeSwitcher/>
                </NavbarItem>
            </NavbarContent>

            <NavbarMenu>
                {menuItems}
            </NavbarMenu>
        </Navbar>)
}